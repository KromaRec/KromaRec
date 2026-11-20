use crate::{accounts, json_body, settings::Store};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use rusqlite::{OptionalExtension, TransactionBehavior, params};
use serde_json::json;
use sha2::{Digest, Sha256};

const SCOPE: &str = "offline_access profile rn rn.accounts rn.accounts.gc rn.api rn.chat rn.clubs rn.commerce rn.match.read rn.match.write rn.notify rn.rooms rn.storage";
const ACCESS_TTL: i64 = 86_400;
const REFRESH_TTL: i64 = 30 * 86_400;

enum Grant {
    Signup(String, i64),
    Password(String, String, String),
    Refresh(String),
}

fn invalid(error: &str, description: &str) -> Response {
    json_body(
        StatusCode::BAD_REQUEST,
        json!({"error":error,"error_description":description}).to_string(),
    )
}

fn hash_password(password: &str) -> rusqlite::Result<String> {
    let salt = SaltString::encode_b64(&accounts::random::<16>()?)
        .map_err(|_| rusqlite::Error::InvalidQuery)?;
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| rusqlite::Error::InvalidQuery)
}

fn verify_password(password: &str, hash: &str) -> bool {
    password.len() <= 128
        && PasswordHash::new(hash).is_ok_and(|hash| {
            Argon2::default()
                .verify_password(password.as_bytes(), &hash)
                .is_ok()
        })
}

pub async fn change_password(store: &Store, request: Request<Body>) -> Response {
    let Some(identity) = store.access_identity(request.headers()) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let fields = match accounts::form(request).await {
        Ok(fields) => fields,
        Err(status) => return status.into_response(),
    };
    let old = fields.get("oldPassword").cloned().unwrap_or_default();
    let new = fields.get("newPassword").cloned().unwrap_or_default();
    if !(8..=128).contains(&new.len()) {
        let error = if new.is_empty() {
            "You must enter a new password."
        } else {
            "Password must contain 8 to 128 bytes."
        };
        return json_body(
            StatusCode::BAD_REQUEST,
            json!({"success":false,"error":error}).to_string(),
        );
    }
    let live = store.live.clone();
    let result = store
        .with_db(move |db| {
            let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
            if !identity.is_current(&tx)? {
                return Ok(StatusCode::UNAUTHORIZED);
            }
            let id = identity.id;
            let Some(current) = tx
                .prepare_cached("SELECT password_hash FROM local_account WHERE id=?1")?
                .query_row([id], |row| row.get::<_, Option<String>>(0))
                .optional()?
            else {
                return Ok(StatusCode::NOT_FOUND);
            };
            if current
                .as_deref()
                .is_some_and(|hash| !verify_password(&old, hash))
            {
                return Ok(StatusCode::BAD_REQUEST);
            }
            tx.prepare_cached("UPDATE local_account SET password_hash=?1, access_generation=access_generation+1 WHERE id=?2")?
                .execute(params![hash_password(&new)?, id])?;
            tx.prepare_cached("DELETE FROM local_refresh WHERE account_id=?1")?
                .execute([id])?;
            tx.commit()?;
            live.revoke(id);
            Ok(StatusCode::OK)
        })
        .await;
    match result {
        Ok(StatusCode::OK) => json_body(StatusCode::OK, r#"{"success":true}"#),
        Ok(StatusCode::BAD_REQUEST) => json_body(
            StatusCode::BAD_REQUEST,
            r#"{"success":false,"error":"Your old password is incorrect."}"#,
        ),
        Ok(status) => status.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn token(store: &Store, request: Request<Body>, signup: bool) -> Response {
    let fields = match accounts::form(request).await {
        Ok(fields) => fields,
        Err(status) => return status.into_response(),
    };
    let get = |name| fields.get(name).map_or("", String::as_str);
    let version = match get("ver") {
        "" => "20230414",
        v if v.len() <= 32 && v.split('.').next() == Some("20230414") => v,
        _ => return invalid("invalid_grant", "Unsupported game version"),
    }
    .to_owned();
    let password = get("password");
    let grant = match get("grant_type") {
        "create_account" if !signup => {
            return invalid("invalid_grant", "Password signup is disabled");
        }
        "create_account" if !(8..=128).contains(&password.len()) => {
            return invalid("invalid_grant", "Password must contain 8 to 128 bytes");
        }
        "create_account" if !get("platform_id").is_empty() || !get("platform_auth").is_empty() => {
            return invalid(
                "unsupported_grant_type",
                "Platform authentication is unavailable",
            );
        }
        "create_account" => {
            Grant::Signup(password.to_owned(), get("platform").parse().unwrap_or(0))
        }
        "password" | "" => Grant::Password(
            get("account_id").trim().into(),
            get("username").trim().into(),
            password.into(),
        ),
        "refresh_token" => Grant::Refresh(get("refresh_token").into()),
        _ => return invalid("unsupported_grant_type", "Unsupported grant"),
    };
    let refresh_failure = matches!(grant, Grant::Refresh(_));
    let secret = store.secret.clone();
    let result = store.with_db(move |db| {
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let now: i64 = tx.query_row("SELECT unixepoch()", [], |row| row.get(0))?;
        let id = match grant {
            Grant::Signup(password, platforms) => {
                let hash = hash_password(&password)?;
                let id = accounts::create(&tx, platforms, Some(hash))?["accountId"].as_i64()
                    .ok_or(rusqlite::Error::InvalidQuery)?;
                crate::sessions::start_orientation(&tx, id)?;
                id
            }
            Grant::Password(id, name, password) => {
                if password.len() > 128 {
                    return Ok(None);
                }
                let credential: Option<(i64, Option<String>)> = if let Ok(id) = id.parse::<i64>() {
                    tx.prepare_cached("SELECT id,password_hash FROM local_account WHERE id=?1")?
                        .query_row([id], |row| Ok((row.get(0)?, row.get(1)?))).optional()?
                } else {
                    tx.prepare_cached("SELECT id,password_hash FROM local_account WHERE username=?1")?
                        .query_row([name], |row| Ok((row.get(0)?, row.get(1)?))).optional()?
                };
                let Some((id, Some(hash))) = credential else { return Ok(None); };
                if !verify_password(&password,&hash) { return Ok(None); }
                id
            }
            Grant::Refresh(token) => {
                let digest = Sha256::digest(token.as_bytes());
                let id = tx.prepare_cached("DELETE FROM local_refresh WHERE digest=?1 AND expires>?2 RETURNING account_id")?
                    .query_row(params![&digest[..], now], |row| row.get(0)).optional()?;
                let Some(id) = id else { return Ok(None); };
                id
            }
        };
        let refresh = URL_SAFE_NO_PAD.encode(accounts::random::<32>()?);
        let digest = Sha256::digest(refresh.as_bytes());
        tx.prepare_cached("DELETE FROM local_refresh WHERE expires<=?1")?.execute([now])?;
        tx.prepare_cached("INSERT INTO local_refresh(digest,account_id,expires) VALUES (?1,?2,?3)")?
            .execute(params![&digest[..], id, now + REFRESH_TTL])?;
        let generation: i64 = tx.prepare_cached("SELECT access_generation FROM local_account WHERE id=?1")?
            .query_row([id], |row| row.get(0))?;
        let claims = json!({"sub":id.to_string(),"iat":now,"nbf":now,"exp":now+ACCESS_TTL,
            "iss":"https://auth.recflare.net","aud":"https://auth.recflare.net","auth_time":now,
            "amr":"cached_login","client_id":"recroom","idp":"local","platform":0,"platform_id":"",
            "rn.ver":version,"rn.plat":0,"role":["gameClient"],"scope":SCOPE.split_ascii_whitespace().collect::<Vec<_>>(),
            "kromarec_access_generation":generation,"jti":URL_SAFE_NO_PAD.encode(accounts::random::<16>()?)});
        let access = sign_claims(&secret, &claims)?;
        tx.commit()?;
        Ok(Some(json!({"access_token":access,"expires_in":ACCESS_TTL,"token_type":"Bearer",
            "refresh_token":refresh,"scope":SCOPE,"key":"8oQ+e+WQaOBPbEcakhqs3dwZZdOmmyDUmJSD9u4AHMY="})))
    }).await;
    match result {
        Ok(Some(body)) => {
            let mut response = json_body(StatusCode::OK, body.to_string());
            response
                .headers_mut()
                .insert("cache-control", "no-store".parse().unwrap());
            response
        }
        Ok(None) if refresh_failure => {
            invalid("invalid_grant", "refresh_token is invalid or expired")
        }
        Ok(None) => invalid("invalid_grant", "invalid account_id or password"),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub(crate) fn sign_claims(secret: &[u8], claims: &serde_json::Value) -> rusqlite::Result<String> {
    let signed = format!(
        "{}.{}",
        URL_SAFE_NO_PAD.encode(br#"{"alg":"HS256","typ":"JWT"}"#),
        URL_SAFE_NO_PAD.encode(claims.to_string())
    );
    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret).map_err(|_| rusqlite::Error::InvalidQuery)?;
    mac.update(signed.as_bytes());
    Ok(format!(
        "{signed}.{}",
        URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
    ))
}
