use crate::{json_body, not_found, settings::Store};
use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
    response::{IntoResponse, Response},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};
use std::time::Duration;

pub const COLUMNS: &str = "id,username,platforms,created_at,identity_flags,personal_pronouns";

pub fn profile(row: &rusqlite::Row<'_>) -> rusqlite::Result<Value> {
    let mut value = dto(
        row.get(0)?,
        row.get::<_, String>(1)?,
        row.get(2)?,
        row.get::<_, String>(3)?,
    );
    value["identityFlags"] = json!(row.get::<_, i64>(4)?);
    value["personalPronouns"] = json!(row.get::<_, i64>(5)?);
    Ok(value)
}

fn dto(id: i64, name: String, platforms: i64, created: String) -> Value {
    json!({"accountId":id,"username":name,"displayName":name,"profileImage":"DefaultProfileImage.jpg",
        "bannerImage":"","displayEmoji":"","isJunior":false,"platforms":platforms,
        "personalPronouns":0,"identityFlags":0,"createdAt":created})
}

pub fn random<const N: usize>() -> rusqlite::Result<[u8; N]> {
    let mut bytes = [0; N];
    getrandom::fill(&mut bytes).map_err(|_| rusqlite::Error::InvalidQuery)?;
    Ok(bytes)
}

pub fn create(db: &Connection, platforms: i64, hash: Option<String>) -> rusqlite::Result<Value> {
    let name = format!("Player{}", URL_SAFE_NO_PAD.encode(random::<12>()?));
    db.query_row(
        "INSERT INTO local_account(id,username,platforms,created_at,password_hash)
         SELECT COALESCE(MAX(id),1)+1,?1,?2,strftime('%Y-%m-%dT%H:%M:%fZ','now'),?3 FROM local_account
         RETURNING id,username,platforms,created_at,identity_flags,personal_pronouns",
        params![name, platforms, hash], profile
    )
}

pub async fn form(
    request: Request<Body>,
) -> Result<std::collections::BTreeMap<String, String>, StatusCode> {
    Ok(form_pairs(request).await?.into_iter().collect())
}

pub(crate) async fn form_pairs(
    request: Request<Body>,
) -> Result<Vec<(String, String)>, StatusCode> {
    let mime = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !mime.starts_with("application/x-www-form-urlencoded") {
        return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }
    let bytes = tokio::time::timeout(Duration::from_secs(10), to_bytes(request.into_body(), 8192))
        .await
        .map_err(|_| StatusCode::REQUEST_TIMEOUT)?
        .map_err(|_| StatusCode::PAYLOAD_TOO_LARGE)?;
    Ok(form_urlencoded::parse(&bytes).into_owned().collect())
}

pub async fn serve(
    store: &Store,
    request: Request<Body>,
    method: Method,
    signup: bool,
) -> Response {
    let path = request.uri().path();
    if method == Method::POST && path == "/account/create" {
        if !signup {
            return StatusCode::FORBIDDEN.into_response();
        }
        let fields = match form(request).await {
            Ok(fields) => fields,
            Err(status) => return status.into_response(),
        };
        let platforms = fields
            .get("platform")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        return match store.with_db(move |db| create(db, platforms, None)).await {
            Ok(value) => json_body(
                StatusCode::OK,
                json!({"success":true,"value":value}).to_string(),
            ),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
    }
    let edit = match (&method, path) {
        (&Method::PUT, "/account/me/bio") => Some(("bio", "bio")),
        (&Method::POST, "/account/me/identityflags") => Some(("identity_flags", "identityFlags")),
        (&Method::PUT, "/account/me/personalpronouns") => {
            Some(("personal_pronouns", "pronounFlags"))
        }
        _ => None,
    };
    if let Some((column, field)) = edit {
        let Some(id) = store.authenticate(request.headers()).await else {
            return StatusCode::UNAUTHORIZED.into_response();
        };
        let Ok(mut fields) = form(request).await else {
            return StatusCode::BAD_REQUEST.into_response();
        };
        let Some(value) = fields.remove(field) else {
            return StatusCode::BAD_REQUEST.into_response();
        };
        let value = if column == "bio" {
            if value.encode_utf16().count() > 255 {
                return StatusCode::BAD_REQUEST.into_response();
            }
            rusqlite::types::Value::Text(value)
        } else {
            let Ok(value) = value.trim().parse::<i64>() else {
                return StatusCode::BAD_REQUEST.into_response();
            };
            rusqlite::types::Value::Integer(value)
        };
        return match store
            .with_db(move |db| {
                db.prepare_cached(&format!("UPDATE local_account SET {column}=?1 WHERE id=?2"))?
                    .execute(params![value, id])
            })
            .await
        {
            Ok(1) => json_body(StatusCode::OK, r#"{"success":true}"#),
            Ok(_) => StatusCode::NOT_FOUND.into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
    }
    if method == Method::GET
        && let Some(raw) = path
            .strip_prefix("/account/")
            .and_then(|s| s.strip_suffix("/bio"))
    {
        let Ok(id) = raw.parse::<i64>() else {
            return StatusCode::BAD_REQUEST.into_response();
        };
        return match store
            .with_db(move |db| {
                db.prepare_cached("SELECT bio FROM local_account WHERE id=?1")?
                    .query_row([id], |row| row.get::<_, String>(0))
                    .optional()
            })
            .await
        {
            Ok(value) => json_body(
                StatusCode::OK,
                json!({"accountId":id,"bio":value.unwrap_or_default()}).to_string(),
            ),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
    }
    if method != Method::GET {
        return not_found();
    }
    let private = path == "/account/me";
    let query = if private {
        let Some(id) = store.authenticate(request.headers()).await else {
            return StatusCode::UNAUTHORIZED.into_response();
        };
        Lookup::Id(id)
    } else if let Some(name) = path.strip_prefix("/account/username/") {
        let Ok(name) = percent_encoding::percent_decode_str(name).decode_utf8() else {
            return StatusCode::BAD_REQUEST.into_response();
        };
        Lookup::Name(name.trim().trim_start_matches('@').to_owned())
    } else if let Some(id) = path
        .strip_prefix("/account/")
        .filter(|id| !id.contains('/') && !matches!(*id, "bulk" | "search"))
    {
        let Ok(id) = id.parse::<i64>() else {
            return StatusCode::BAD_REQUEST.into_response();
        };
        Lookup::Id(id)
    } else {
        return not_found();
    };
    let found = store
        .with_db(move |db| match query {
            Lookup::Id(id) => {
                let mut statement =
                    db.prepare_cached(&format!("SELECT {COLUMNS} FROM local_account WHERE id=?1"))?;
                let value = statement.query_row([id], profile).optional()?;
                match value {
                    Some(value) => Ok((StatusCode::OK, value)),
                    None => {
                        let now: String = db.query_row(
                            "SELECT strftime('%Y-%m-%dT%H:%M:%fZ','now')",
                            [],
                            |row| row.get(0),
                        )?;
                        Ok((StatusCode::OK, dto(id, format!("Player{id}"), 0, now)))
                    }
                }
            }
            Lookup::Name(name) => {
                let mut statement = db.prepare_cached(&format!(
                    "SELECT {COLUMNS} FROM local_account WHERE username=?1"
                ))?;
                Ok(match statement.query_row([&name], profile).optional()? {
                    Some(value) => (StatusCode::OK, value),
                    None => (
                        StatusCode::NOT_FOUND,
                        json!({"error":format!("There's no player called @{name}.")}),
                    ),
                })
            }
        })
        .await;
    match found {
        Ok((status, mut value)) => {
            if private {
                value["email"] = json!("");
                value["birthday"] = json!("1904-01-01T00:00:00.000Z");
                value["availableUsernameChanges"] = json!(3);
                value["isDeleted"] = json!(false);
            }
            json_body(status, value.to_string())
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

enum Lookup {
    Id(i64),
    Name(String),
}
