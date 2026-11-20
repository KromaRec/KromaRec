use crate::{accounts, json_body, not_found, settings::Store};
use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    response::{IntoResponse, Response},
};
use rusqlite::{OptionalExtension, params};
use serde_json::json;

pub async fn serve(
    store: Option<&Store>,
    hub: &crate::notify::Hub,
    region: &str,
    request: Request<Body>,
) -> Response {
    if request.method() != Method::POST {
        return not_found();
    }
    let route = match request.uri().path() {
        "/player/login" => "login",
        "/player/exclusivelogin" => "exclusive",
        "/player/verifylock" => "verify",
        "/player/logout" => "logout",
        "/player/heartbeat" => "heartbeat",
        _ => return not_found(),
    };
    let Some((store, identity)) = store.and_then(|store| {
        store
            .access_identity(request.headers())
            .map(|identity| (store, identity))
    }) else {
        return if matches!(route, "heartbeat" | "exclusive" | "verify") {
            StatusCode::UNAUTHORIZED
        } else {
            StatusCode::OK
        }
        .into_response();
    };
    let lock = accounts::form(request).await.ok().and_then(|fields| {
        fields
            .into_iter()
            .find(|(key, _)| key.eq_ignore_ascii_case("LoginLock"))
            .map(|(_, value)| value)
            .filter(|value| !value.is_empty() && value.len() <= 128)
    });
    let id = identity.id;
    let logout = route == "logout";
    let push_region = region.to_owned();
    let region = region.to_owned();
    let live = store.live.clone();
    let result = store.with_db(move |db| {
        let tx = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        if !identity.is_current(&tx)? || !crate::rooms::account_exists(&tx, id)? {
            return Ok((StatusCode::UNAUTHORIZED,None,false));
        }
        let now: i64 = tx.query_row("SELECT unixepoch()", [], |r|r.get(0))?;
        let mut online = false;
        let lock_present = lock.is_some();
        let mut changed_presence = false;
        match route {
            "verify" => {
                let verified: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM lobby_presence
                    WHERE account_id=?1 AND login_lock=?2 AND expires>?3)", params![id,lock,now],|r|r.get(0))?;
                return Ok((if verified {StatusCode::OK} else {StatusCode::CONFLICT},None,false));
            }
            "login" | "exclusive" => if let Some(lock) = lock {
                tx.prepare_cached("DELETE FROM lobby_presence WHERE expires<=?1")?.execute([now])?;
                tx.prepare_cached("INSERT INTO lobby_presence(account_id,login_lock,expires) VALUES(?1,?2,?3)
                    ON CONFLICT(account_id) DO UPDATE SET login_lock=excluded.login_lock, expires=excluded.expires")?
                    .execute(params![id,lock,now+900])?;
            },
            "logout" if crate::sessions::in_orientation(&tx,id)? => {}
            "logout" => {
                if let Some(lock) = &lock {
                    let superseded: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM lobby_presence
                        WHERE account_id=?1 AND expires>?2 AND login_lock<>?3)",
                        params![id,now,lock], |r| r.get(0))?;
                    if superseded { return Ok((StatusCode::OK,None,false)); }
                }
                changed_presence = true;
                tx.prepare_cached("UPDATE local_account SET photon_generation=photon_generation+1 WHERE id=?1")?
                    .execute([id])?;
                crate::sessions::leave(&tx,id)?;
                tx.prepare_cached("DELETE FROM lobby_presence WHERE account_id=?1")?.execute([id])?;
            }
            _ => {
                let existing = tx.prepare_cached("SELECT login_lock,expires FROM lobby_presence WHERE account_id=?1 AND expires>?2")?
                    .query_row(params![id,now], |row| Ok((row.get::<_,String>(0)?,row.get::<_,i64>(1)?))).optional()?;
                if existing.is_none() { crate::sessions::refresh(&tx,id)?; }
                if let Some((active, expiry)) = existing {
                    if lock.is_some_and(|posted| posted != active) {
                        return Ok((StatusCode::OK,None,false));
                    }
                    online = true;
                    crate::sessions::refresh(&tx,id)?;
                    if expiry-now <= 300 {
                        tx.prepare_cached("UPDATE lobby_presence SET expires=?2 WHERE account_id=?1")?.execute(params![id,now+900])?;
                    }
                }
            }
        }
        let room = if route == "heartbeat" { crate::sessions::current_dto(&tx,id,&region)? } else { serde_json::Value::Null };
        tx.commit()?;
        if changed_presence { live.revoke(id); }
        if route == "exclusive" {
            return Ok((StatusCode::OK,Some(json!({"errorCode": if lock_present {0} else {-1}})),false));
        }
        Ok((StatusCode::OK,(route == "heartbeat").then(|| json!({"appVersion":"20230414","deviceClass":0,"errorCode":0,
            "isOnline":online || !room.is_null(),"playerId":id,"roomInstance":room,"statusVisibility":0,"vrMovementMode":1,
            "platform":0,"photonAuthToken":null,"photonRealtimeAppId":null,"photonVoiceAppId":null,
            "photonChatAppId":null,"photonRegion":null,"photonRoomId":null,"voiceConnectionInfo":null,
            "voiceServerId":null,"experiments":null})),changed_presence))
    }).await;
    if logout && matches!(&result, Ok((_, _, true))) {
        crate::sessions::push_presence(store, hub, id, push_region).await;
    }
    match result {
        Ok((status, Some(value), _)) => json_body(status, value.to_string()),
        Ok((status, None, _)) => status.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
