use axum::{
    body::{Body, to_bytes},
    http::{HeaderMap, Method, Request, StatusCode, header},
    response::{IntoResponse, Response},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::Sha256;
use std::{
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const BODY_LIMIT: usize = 32 * 1024;
const DEFAULTS: [(&str, &str); 3] = [
    ("Recroom.OOBE", "77"),
    ("TUTORIAL_COMPLETE_MASK", "11"),
    ("FIRST_TIME_IN_FLAGS", "0"),
];

pub struct Store {
    db: Arc<Mutex<Connection>>,
    gate: Arc<tokio::sync::Semaphore>,
    pub(crate) live: Arc<crate::live::Registry>,
    pub(crate) secret: Vec<u8>,
}

#[derive(Deserialize)]
struct Claims {
    sub: String,
    #[serde(rename = "rn.env")]
    photon_environment: Option<String>,
    exp: Option<f64>,
    kromarec_access_generation: Option<i64>,
    nbf: Option<f64>,
    iat: Option<f64>,
}

pub(crate) struct AccessIdentity {
    pub(crate) id: i64,
    pub(crate) expires: Option<u64>,
    generation: Option<i64>,
}

impl AccessIdentity {
    pub(crate) fn is_current(&self, db: &Connection) -> rusqlite::Result<bool> {
        let current = db
            .prepare_cached("SELECT access_generation FROM local_account WHERE id=?1")?
            .query_row([self.id], |row| row.get::<_, i64>(0))
            .optional()?;
        // Keep externally signed migration fixtures and pre-upgrade HTTP tokens compatible.
        Ok(match self.generation {
            Some(generation) => current == Some(generation),
            None => current.unwrap_or(0) == 0,
        })
    }
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct Entry {
    player_id: i64,
    key: String,
    value: String,
}

enum Operation {
    Read,
    Write(Vec<(String, String)>),
    Delete(Vec<String>),
}

impl Store {
    pub fn open(path: &str, secret: Vec<u8>) -> rusqlite::Result<Self> {
        let mut db = Connection::open(path)?;
        db.busy_timeout(Duration::from_secs(2))?;
        db.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; PRAGMA cache_size=-512;",
        )?;
        db.set_prepared_statement_cache_capacity(12);
        let tx = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        match tx.query_row("PRAGMA user_version", [], |row| row.get::<_, u32>(0))? {
            0 => tx.execute_batch(
                "CREATE TABLE player_settings (
                    sequence INTEGER PRIMARY KEY,
                    player_id INTEGER NOT NULL,
                    name TEXT NOT NULL,
                    value TEXT NOT NULL,
                    UNIQUE(player_id, name)
                );
                PRAGMA user_version=1;",
            )?,
            1..=9 => {}
            _ => return Err(rusqlite::Error::InvalidQuery),
        }
        if tx.query_row("PRAGMA user_version", [], |row| row.get::<_, u32>(0))? == 1 {
            tx.execute_batch(
                "CREATE TABLE local_account (
                   id INTEGER PRIMARY KEY, username TEXT NOT NULL COLLATE NOCASE UNIQUE,
                   platforms INTEGER NOT NULL, created_at TEXT NOT NULL, password_hash TEXT
                 );
                 CREATE TABLE local_refresh (
                   digest BLOB PRIMARY KEY, account_id INTEGER NOT NULL REFERENCES local_account(id),
                   expires INTEGER NOT NULL
                 );
                 CREATE INDEX local_refresh_expiry ON local_refresh(expires);
                 PRAGMA user_version=2;",
            )?;
        }
        if tx.query_row("PRAGMA user_version", [], |row| row.get::<_, u32>(0))? == 2 {
            tx.execute_batch(include_str!("../migrations/003.sql"))?;
        }
        if tx.query_row("PRAGMA user_version", [], |row| row.get::<_, u32>(0))? == 3 {
            tx.execute_batch(include_str!("../migrations/004.sql"))?;
        }
        if tx.query_row("PRAGMA user_version", [], |row| row.get::<_, u32>(0))? == 4 {
            tx.execute_batch(include_str!("../migrations/005.sql"))?;
        }
        if tx.query_row("PRAGMA user_version", [], |row| row.get::<_, u32>(0))? == 5 {
            tx.execute_batch(include_str!("../migrations/006.sql"))?;
        }
        if tx.query_row("PRAGMA user_version", [], |row| row.get::<_, u32>(0))? == 6 {
            tx.execute_batch(include_str!("../migrations/007.sql"))?;
        }
        if tx.query_row("PRAGMA user_version", [], |row| row.get::<_, u32>(0))? == 7 {
            tx.execute_batch(include_str!("../migrations/008.sql"))?;
        }
        if tx.query_row("PRAGMA user_version", [], |row| row.get::<_, u32>(0))? == 8 {
            tx.execute_batch(include_str!("../migrations/009.sql"))?;
        }
        tx.commit()?;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            gate: Arc::new(tokio::sync::Semaphore::new(1)),
            live: Arc::default(),
            secret,
        })
    }

    pub(crate) async fn authenticate(&self, headers: &HeaderMap) -> Option<i64> {
        let identity = self.access_identity(headers)?;
        self.with_db(move |db| Ok(identity.is_current(db)?.then_some(identity.id)))
            .await
            .ok()
            .flatten()
    }

    pub(crate) fn access_identity(&self, headers: &HeaderMap) -> Option<AccessIdentity> {
        let authorization = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
        let (scheme, token) = authorization.split_once(' ')?;
        if !scheme.eq_ignore_ascii_case("bearer") || token.len() > 8192 {
            return None;
        }
        let claims: Claims = self.verify_signed(token)?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs() as f64;
        if claims.photon_environment.is_some()
            || claims.exp.is_some_and(|exp| exp != 0.0 && exp <= now)
            || claims.nbf.is_some_and(|nbf| nbf > now)
            || claims.iat.is_some_and(|iat| iat > now)
            || !claims.sub.bytes().all(|byte| byte.is_ascii_digit())
            || claims.sub.starts_with('0')
        {
            return None;
        }
        let id = claims.sub.parse::<i64>().ok()?;
        (id <= 9_007_199_254_740_991).then_some(AccessIdentity {
            id,
            expires: claims
                .exp
                .filter(|value| *value > 0.0)
                .map(|value| value.ceil() as u64),
            generation: claims.kromarec_access_generation,
        })
    }

    pub(crate) fn verify_signed<T: serde::de::DeserializeOwned>(&self, token: &str) -> Option<T> {
        if token.len() > 8192 {
            return None;
        }
        let (signed, signature) = token.rsplit_once('.')?;
        let (metadata, payload) = signed.split_once('.')?;
        let metadata: Value =
            serde_json::from_slice(&URL_SAFE_NO_PAD.decode(metadata).ok()?).ok()?;
        if metadata.get("alg")?.as_str()? != "HS256" {
            return None;
        }
        let mut mac = Hmac::<Sha256>::new_from_slice(&self.secret).ok()?;
        mac.update(signed.as_bytes());
        mac.verify_slice(&URL_SAFE_NO_PAD.decode(signature).ok()?)
            .ok()?;
        serde_json::from_slice(&URL_SAFE_NO_PAD.decode(payload).ok()?).ok()
    }

    pub(crate) async fn with_db<T: Send + 'static>(
        &self,
        work: impl FnOnce(&mut Connection) -> rusqlite::Result<T> + Send + 'static,
    ) -> rusqlite::Result<T> {
        let permit = self
            .gate
            .clone()
            .acquire_owned()
            .await
            .expect("database gate stays open");
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            // A timed-out request still occupies the database until its blocking work ends.
            let _permit = permit;
            work(&mut db.lock().expect("database lock"))
        })
        .await
        .map_err(|_| rusqlite::Error::InvalidQuery)?
    }

    pub async fn handle(&self, request: Request<Body>, head: bool) -> Response {
        let (parts, body) = request.into_parts();
        let Some(id) = self.authenticate(&parts.headers).await else {
            return StatusCode::UNAUTHORIZED.into_response();
        };
        let op = if head || parts.method == Method::GET {
            Operation::Read
        } else {
            let Ok(Ok(bytes)) =
                tokio::time::timeout(Duration::from_secs(10), to_bytes(body, BODY_LIMIT)).await
            else {
                return StatusCode::PAYLOAD_TOO_LARGE.into_response();
            };
            let mime = parts
                .headers
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            let deleting = parts.method == Method::DELETE;
            let values = parse_pairs(&bytes, mime, deleting).await;
            if deleting {
                let mut keys: Vec<_> = values.into_iter().map(|(key, _)| key).collect();
                if keys.is_empty()
                    && !mime.contains("application/json")
                    && let Some((_, key)) =
                        form_urlencoded::parse(parts.uri.query().unwrap_or("").as_bytes())
                            .find(|(name, value)| name == "key" && !value.is_empty())
                {
                    keys.push(key.into_owned());
                }
                if keys.is_empty() {
                    return StatusCode::OK.into_response();
                }
                Operation::Delete(keys)
            } else {
                let mut values = values;
                values.retain(|(key, _)| {
                    key != "MakerPenSessionCount" && !key.starts_with("Growth.YIR.")
                });
                if values.is_empty() {
                    return StatusCode::OK.into_response();
                }
                Operation::Write(values)
            }
        };
        match self.with_db(move |db| execute(db, id, op)).await {
            Ok(Some(entries)) => match serde_json::to_vec(&entries) {
                Ok(bytes) => crate::json_body(StatusCode::OK, bytes),
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            },
            Ok(None) => StatusCode::OK.into_response(),
            _ => {
                eprintln!("settings database operation failed");
                crate::json_body(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    r#"{"success":false,"error":{"message":"internal server error"}}"#,
                )
            }
        }
    }
}

fn execute(db: &mut Connection, id: i64, op: Operation) -> rusqlite::Result<Option<Vec<Entry>>> {
    let tx = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let output = match op {
        Operation::Read => {
            let mut read = tx.prepare_cached(
                "SELECT name, value FROM player_settings WHERE player_id=?1 ORDER BY sequence",
            )?;
            let mut entries = read
                .query_map([id], |row| {
                    Ok(Entry {
                        player_id: id,
                        key: row.get(0)?,
                        value: row.get(1)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            if entries.is_empty() {
                let mut insert = tx.prepare_cached(
                    "INSERT INTO player_settings(player_id,name,value) VALUES (?1,?2,?3)",
                )?;
                for (key, value) in DEFAULTS {
                    insert.execute(params![id, key, value])?;
                    entries.push(Entry {
                        player_id: id,
                        key: key.into(),
                        value: value.into(),
                    });
                }
            }
            Some(entries)
        }
        Operation::Write(pairs) => {
            let mut write = tx.prepare_cached(
                "INSERT INTO player_settings(player_id,name,value) VALUES (?1,?2,?3)
                 ON CONFLICT(player_id,name) DO UPDATE SET value=excluded.value
                 WHERE value != excluded.value",
            )?;
            for (key, value) in pairs {
                write.execute(params![id, key, value])?;
            }
            None
        }
        Operation::Delete(keys) => {
            let mut delete =
                tx.prepare_cached("DELETE FROM player_settings WHERE player_id=?1 AND name=?2")?;
            for key in keys {
                delete.execute(params![id, key])?;
            }
            None
        }
    };
    tx.commit()?;
    Ok(output)
}

async fn parse_pairs(bytes: &[u8], mime: &str, deleting: bool) -> Vec<(String, String)> {
    if mime.contains("application/json") {
        let items = match serde_json::from_slice::<Value>(bytes) {
            Ok(Value::Array(items)) => items,
            Ok(item) => vec![item],
            Err(_) => return Vec::new(),
        };
        return items
            .into_iter()
            .filter_map(|item| {
                let key = if deleting && item.is_string() {
                    item.as_str()
                } else {
                    item.get("key")
                        .or_else(|| item.get("Key"))
                        .and_then(Value::as_str)
                }
                .filter(|s| !s.is_empty())?
                .to_owned();
                let value = match item.get("value").or_else(|| item.get("Value")) {
                    Some(Value::String(value)) => value.clone(),
                    Some(value @ (Value::Bool(_) | Value::Number(_))) => value.to_string(),
                    _ => String::new(),
                };
                Some((key, value))
            })
            .collect();
    }
    let mut result = (String::new(), String::new());
    if let Ok(boundary) = multer::parse_boundary(mime) {
        let stream = axum::body::Body::from(bytes.to_vec()).into_data_stream();
        let mut multipart = multer::Multipart::new(stream, boundary);
        loop {
            match multipart.next_field().await {
                Ok(Some(field)) if field.file_name().is_none() => match field.name() {
                    Some("key") => result.0 = field.text().await.unwrap_or_default(),
                    Some("value") => result.1 = field.text().await.unwrap_or_default(),
                    _ => {}
                },
                Ok(Some(_)) => {}
                Ok(None) => break,
                Err(_) => return Vec::new(),
            }
        }
    } else if deleting || mime.contains("application/x-www-form-urlencoded") {
        for (name, value) in form_urlencoded::parse(bytes) {
            match name.as_ref() {
                "key" => result.0 = value.into_owned(),
                "value" => result.1 = value.into_owned(),
                _ => {}
            }
        }
    }
    if result.0.is_empty() {
        Vec::new()
    } else {
        vec![result]
    }
}

#[cfg(test)]
mod tests {
    use super::Store;
    use std::sync::Arc;
    use tokio::sync::oneshot;

    #[tokio::test]
    async fn cancelled_request_keeps_database_slot_until_blocking_work_finishes() {
        let store = Arc::new(Store::open(":memory:", vec![0; 32]).unwrap());
        let (started, wait_started) = oneshot::channel();
        let (finish, wait_finish) = oneshot::channel();
        let request = tokio::spawn({
            let store = Arc::clone(&store);
            async move {
                store
                    .with_db(|_| {
                        started.send(()).unwrap();
                        // Dropping the sender also releases the worker if the test fails.
                        let _ = wait_finish.blocking_recv();
                        Ok(())
                    })
                    .await
            }
        });
        wait_started.await.unwrap();
        request.abort();
        assert!(request.await.unwrap_err().is_cancelled());
        assert_eq!(store.gate.available_permits(), 0);
        finish.send(()).unwrap();
        assert_eq!(
            store
                .with_db(|db| db.query_row("SELECT 1", [], |r| r.get::<_, i64>(0)))
                .await
                .unwrap(),
            1
        );
    }
}
