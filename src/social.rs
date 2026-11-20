use crate::{json_body, not_found, settings::Store};
use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
    response::{IntoResponse, Response},
};
use rusqlite::{OptionalExtension, Transaction, TransactionBehavior, params};
use serde_json::{Value, json};
use std::time::Duration;

enum Operation {
    List,
    Mutual(i64),
    Inbox,
    Online,
    Delete(Vec<i64>),
    Change(i64, Action),
}

enum Action {
    Request,
    Accept,
    Remove,
    Add,
    Flag(&'static str, bool),
}

fn integer(value: &Value) -> Option<i64> {
    value
        .as_i64()
        .or_else(|| value.as_str()?.trim().parse().ok())
}

pub async fn serve(store: &Store, request: Request<Body>, method: &Method) -> Response {
    let path = request.uri().path();
    let action = match path {
        "/api/relationships/v2/sendfriendrequest" => Some(Action::Request),
        "/api/relationships/v2/acceptfriendrequest" => Some(Action::Accept),
        "/api/relationships/v2/removefriend" => Some(Action::Remove),
        "/api/relationships/v2/addfriend" => Some(Action::Add),
        "/api/relationships/v1/ignore" => Some(Action::Flag("ignored", true)),
        "/api/relationships/v1/unignore" => Some(Action::Flag("ignored", false)),
        "/api/relationships/v1/mute" => Some(Action::Flag("muted", true)),
        "/api/relationships/v1/unmute" => Some(Action::Flag("muted", false)),
        "/api/relationships/v1/favorite" => Some(Action::Flag("favorited", true)),
        "/api/relationships/v1/unfavorite" => Some(Action::Flag("favorited", false)),
        _ => None,
    };
    let Some(id) = store.authenticate(request.headers()).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let op = match (method, path) {
        (&Method::GET, "/api/relationships/v2/get") => Operation::List,
        (&Method::GET, "/api/relationships/mutualfriends") => {
            let peer = form_urlencoded::parse(request.uri().query().unwrap_or("").as_bytes())
                .find(|(key, _)| key == "id")
                .and_then(|(_, value)| value.parse().ok())
                .unwrap_or(0);
            Operation::Mutual(peer)
        }
        (&Method::POST, "/api/messages/v1/friendOnlineStatus") => Operation::Online,
        (&Method::GET, "/api/messages/v2/get") => Operation::Inbox,
        (&Method::POST, "/api/messages/v3/delete") => {
            let body = match read_body(request).await {
                Ok(body) => body,
                Err(status) => return status.into_response(),
            };
            let value: Value = serde_json::from_slice(&body).unwrap_or_default();
            let ids = value
                .as_array()
                .or_else(|| value.get("MessageIds")?.as_array())
                .into_iter()
                .flatten()
                .filter_map(integer)
                .collect();
            Operation::Delete(ids)
        }
        (&Method::GET | &Method::POST, _) if action.is_some() => {
            let target = match target(request).await {
                Ok(Some(peer)) if peer > 0 && peer != id && peer <= 9_007_199_254_740_991 => peer,
                Ok(_) => {
                    return json_body(StatusCode::BAD_REQUEST, r#"{"error":"invalid player id"}"#);
                }
                Err(status) => return status.into_response(),
            };
            Operation::Change(target, action.expect("matched action"))
        }
        _ => return not_found(),
    };
    match store
        .with_db(move |db| {
            let behavior = match op {
                Operation::Change(..) | Operation::Delete(_) => TransactionBehavior::Immediate,
                _ => TransactionBehavior::Deferred,
            };
            let tx = db.transaction_with_behavior(behavior)?;
            let value = execute(&tx, id, op)?;
            tx.commit()?;
            Ok(value)
        })
        .await
    {
        Ok(Some(value)) => json_body(StatusCode::OK, value.to_string()),
        Ok(None) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn read_body(request: Request<Body>) -> Result<axum::body::Bytes, StatusCode> {
    tokio::time::timeout(Duration::from_secs(10), to_bytes(request.into_body(), 8192))
        .await
        .map_err(|_| StatusCode::REQUEST_TIMEOUT)?
        .map_err(|_| StatusCode::PAYLOAD_TOO_LARGE)
}

async fn target(request: Request<Body>) -> Result<Option<i64>, StatusCode> {
    let query: std::collections::BTreeMap<_, _> =
        form_urlencoded::parse(request.uri().query().unwrap_or("").as_bytes()).collect();
    if let Some(peer) = query
        .get("playerId")
        .or_else(|| query.get("id"))
        .and_then(|v| v.parse().ok())
    {
        return Ok(Some(peer));
    }
    let mime = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|s| s.to_str().ok())
        .unwrap_or("");
    let is_json = mime.contains("application/json");
    let is_form = mime.starts_with("application/x-www-form-urlencoded");
    let body = read_body(request).await?;
    if is_json {
        let value: Value = serde_json::from_slice(&body).unwrap_or_default();
        return Ok(value
            .get("PlayerId")
            .or_else(|| value.get("playerId"))
            .or_else(|| value.get("Id"))
            .and_then(integer));
    }
    Ok(is_form
        .then(|| {
            form_urlencoded::parse(&body)
                .find(|(key, _)| matches!(key.as_ref(), "PlayerId" | "playerId" | "Id"))
                .and_then(|(_, value)| value.parse().ok())
        })
        .flatten())
}

fn relationship(row: &rusqlite::Row<'_>) -> rusqlite::Result<Value> {
    Ok(
        json!({"PlayerID":row.get::<_,i64>(0)?,"RelationshipType":row.get::<_,i64>(1)?,
        "Favorited":row.get::<_,i64>(2)?,"Ignored":row.get::<_,i64>(3)?,"Muted":row.get::<_,i64>(4)?}),
    )
}

fn execute(tx: &Transaction<'_>, id: i64, op: Operation) -> rusqlite::Result<Option<Value>> {
    let result = match op {
        Operation::List => {
            let mut statement = tx.prepare_cached("SELECT peer,kind,favorited,ignored,muted FROM relationship WHERE owner=?1 ORDER BY peer")?;
            Value::Array(
                statement
                    .query_map([id], relationship)?
                    .collect::<rusqlite::Result<_>>()?,
            )
        }
        Operation::Mutual(peer) if peer <= 0 || peer == id => json!([]),
        Operation::Mutual(peer) => {
            let mut statement = tx.prepare_cached(
                "SELECT a.id,a.username FROM relationship x
                 JOIN relationship y ON y.peer=x.peer AND y.owner=?2 AND y.kind=3
                 JOIN local_account a ON a.id=x.peer
                 WHERE x.owner=?1 AND x.kind=3 ORDER BY a.id LIMIT 100",
            )?;
            Value::Array(statement.query_map(params![id,peer], |row| Ok(json!({
                "AccountId":row.get::<_,i64>(0)?,"Username":row.get::<_,String>(1)?,
                "DisplayName":row.get::<_,String>(1)?,"ProfileImage":"DefaultProfileImage.jpg"
            })))?.collect::<rusqlite::Result<_>>()?)
        }
        Operation::Online => {
            let count: i64 = tx
                .prepare_cached(
                    "SELECT COUNT(*) FROM relationship r WHERE r.owner=?1 AND r.kind=3 AND r.ignored=0 AND (
                EXISTS(SELECT 1 FROM lobby_presence p WHERE p.account_id=r.peer AND p.expires>unixepoch())
                OR EXISTS(SELECT 1 FROM room_location l WHERE l.player=r.peer AND l.expires>unixepoch()))",
                )?
                .query_row([id], |row| row.get(0))?;
            json!({"success":true,"value":{"FriendsOnlineCount":count}})
        }
        Operation::Inbox => {
            let mut statement = tx.prepare_cached("SELECT id,sender,kind,sent,data,room_id FROM social_inbox WHERE recipient=?1 ORDER BY id DESC LIMIT 20")?;
            Value::Array(statement.query_map([id], |row| Ok(json!({
                "Id":row.get::<_,i64>(0)?,"FromPlayerId":row.get::<_,i64>(1)?,"ToPlayerId":id,
                "Type":row.get::<_,i64>(2)?,"SentTime":row.get::<_,String>(3)?,
                "Data":row.get::<_,Option<String>>(4)?,"RoomId":row.get::<_,Option<i64>>(5)?,"PlayerEventId":null
            })))?.collect::<rusqlite::Result<_>>()?)
        }
        Operation::Delete(ids) => {
            let mut statement =
                tx.prepare_cached("DELETE FROM social_inbox WHERE recipient=?1 AND id=?2")?;
            for message in ids {
                statement.execute(params![id, message])?;
            }
            return Ok(None);
        }
        Operation::Change(peer, action) => change(tx, id, peer, action)?,
    };
    Ok(Some(result))
}

fn change(tx: &Transaction<'_>, id: i64, peer: i64, action: Action) -> rusqlite::Result<Value> {
    let previous: Option<i64> = tx
        .prepare_cached("SELECT kind FROM relationship WHERE owner=?1 AND peer=?2")?
        .query_row(params![id, peer], |row| row.get(0))
        .optional()?;
    if let Action::Flag(column, set) = action {
        put_pair(tx, id, peer, previous.unwrap_or(0))?;
        tx.prepare_cached(&format!(
            "UPDATE relationship SET {column}=?3 WHERE owner=?1 AND peer=?2"
        ))?
        .execute(params![id, peer, set])?;
        return Ok(json!({"Success":true,"Message":""}));
    }
    let (next, message) = match (action, previous.unwrap_or(0)) {
        (Action::Request, 0) => (Some(1), Some(4)),
        (Action::Request | Action::Accept, 2) => (Some(3), Some(40)),
        (Action::Remove, _) => (previous.map(|_| 0), None),
        (Action::Add, kind) if kind != 3 => (Some(3), None),
        _ => (None, None),
    };
    if let Some(kind) = next {
        put_pair(tx, id, peer, kind)?;
    }
    if let Some(kind) = message {
        tx.prepare_cached("INSERT INTO social_inbox(sender,recipient,kind) VALUES(?1,?2,?3)")?
            .execute(params![id, peer, kind])?;
        tx.prepare_cached(
            "DELETE FROM social_inbox WHERE recipient=?1 AND id < (
            SELECT id FROM social_inbox WHERE recipient=?1 ORDER BY id DESC LIMIT 1 OFFSET 19)",
        )?
        .execute([peer])?;
    }
    Ok(tx
        .prepare_cached(
            "SELECT peer,kind,favorited,ignored,muted FROM relationship WHERE owner=?1 AND peer=?2",
        )?
        .query_row(params![id, peer], relationship)
        .optional()?
        .unwrap_or_else(
            || json!({"PlayerID":peer,"RelationshipType":0,"Favorited":0,"Ignored":0,"Muted":0}),
        ))
}

fn put_pair(tx: &Transaction<'_>, id: i64, peer: i64, kind: i64) -> rusqlite::Result<()> {
    let other = match kind {
        1 => 2,
        2 => 1,
        _ => kind,
    };
    let mut write = tx.prepare_cached(
        "INSERT INTO relationship(owner,peer,kind) VALUES(?1,?2,?3)
        ON CONFLICT(owner,peer) DO UPDATE SET kind=excluded.kind WHERE kind<>excluded.kind",
    )?;
    write.execute(params![id, peer, kind])?;
    write.execute(params![peer, id, other])?;
    Ok(())
}
