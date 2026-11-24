use crate::{App, json_body, not_found};
use axum::{
    http::{HeaderMap, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use rusqlite::{OptionalExtension, params};
use serde_json::{Value, json};
use std::collections::HashSet;

pub fn segment<'a>(path: &'a str, prefix: &str) -> Option<&'a str> {
    path.strip_prefix(prefix)
        .filter(|s| !s.is_empty() && !s.contains('/'))
}

fn query(uri: &Uri, key: &str) -> Option<String> {
    form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.into_owned())
}

fn ok(value: Value) -> Response {
    json_body(StatusCode::OK, value.to_string())
}

fn unavailable() -> Response {
    StatusCode::SERVICE_UNAVAILABLE.into_response()
}

pub fn sections(uri: &Uri) -> Response {
    let ids: HashSet<_> = form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
        .filter(|(k, _)| k == "id")
        .map(|(_, v)| v)
        .collect();
    if ids.is_empty() {
        return json_body(StatusCode::OK, "[]");
    }
    let mut rows: Vec<Value> =
        serde_json::from_str(include_str!("../compat/catalogue/sections.json"))
            .expect("bundled section catalogue");
    rows.retain(|r| r["id"].as_str().is_some_and(|id| ids.contains(id)));
    ok(Value::Array(rows))
}

pub async fn account_feature(app: &App, headers: &HeaderMap, channel: Option<&str>) -> Response {
    let Some(store) = &app.settings else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    if store.authenticate(headers).await.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    match channel {
        None => ok(
            json!({"success":false,"error_id":"AI.RoomDoesNotSupportGameAI",
                          "error":"This room does not support Rec Room Game AI"}),
        ),
        Some("None" | "Announcements" | "JustInTimeTutorials") => json_body(StatusCode::OK, "[]"),
        Some(_) => StatusCode::BAD_REQUEST.into_response(),
    }
}

// Public reads expose only the pinned DTO fields, even for imported records.
fn stored_object(data: &str, fields: &[&str]) -> Response {
    let Ok(Value::Object(mut obj)) = serde_json::from_str(data) else {
        return unavailable();
    };
    obj.retain(|key, _| fields.contains(&key.as_str()));
    ok(Value::Object(obj))
}

pub async fn club(app: &App, id: &str) -> Response {
    if !id.bytes().all(|b| b.is_ascii_digit()) {
        return not_found();
    }
    let Ok(id) = id.parse::<i64>() else {
        return not_found();
    };
    let Some(store) = &app.settings else {
        return unavailable();
    };
    match store
        .with_db(move |db| {
            db.query_row("SELECT data FROM club WHERE club_id=?1", [id], |r| {
                r.get::<_, String>(0)
            })
            .optional()
        })
        .await
    {
        Ok(Some(data)) => stored_object(
            &data,
            &[
                "ClubId",
                "Name",
                "Description",
                "Category",
                "Visibility",
                "Joinability",
                "AllowJuniors",
                "MainImageName",
                "ClubType",
                "ClubhouseRoomId",
                "CreatorAccountId",
                "IsRRO",
                "MinLevel",
                "State",
                "MemberCount",
            ],
        ),
        Ok(None) => not_found(),
        Err(_) => unavailable(),
    }
}

fn number(s: &str) -> Option<i64> {
    let n = s.trim().parse::<f64>().ok()?;
    n.is_finite().then_some(n as i64)
}

pub async fn comments(app: &App, id: &str, uri: &Uri) -> Response {
    let Some(room) = number(id) else {
        return json_body(StatusCode::OK, "[]");
    };
    let Some(store) = &app.settings else {
        return unavailable();
    };
    let int = |k| query(uri, k).as_deref().and_then(number);
    let (sub, min, count) = (
        int("subRoomId"),
        int("minId").unwrap_or(-1),
        int("count").unwrap_or(100).clamp(1, 500),
    );
    let rows = store.with_db(move |db| {
        let mut stmt = db.prepare_cached("SELECT comment_id,subroom_id,player_id,created_at,message,style,
          position_x,position_y,position_z FROM room_comment WHERE room_id=?1 AND comment_id>?2
          AND (?3 IS NULL OR subroom_id=?3) ORDER BY comment_id DESC LIMIT ?4")?;
        stmt.query_map(params![room,min,sub,count], |r| Ok(json!({
            "CommentId":r.get::<_,i64>(0)?, "RoomId":room, "SubRoomId":r.get::<_,i64>(1)?,
            "AccountId":r.get::<_,i64>(2)?, "CreatedAt":r.get::<_,String>(3)?, "Message":r.get::<_,String>(4)?,
            "Style":r.get::<_,i64>(5)?, "Unread":true, "PositionX":r.get::<_,f64>(6)?,
            "PositionY":r.get::<_,f64>(7)?, "PositionZ":r.get::<_,f64>(8)?
        })))?.collect::<rusqlite::Result<Vec<_>>>()
    }).await;
    match rows {
        Ok(rows) => ok(Value::Array(rows)),
        Err(_) => unavailable(),
    }
}

fn parsed_int(text: &str) -> Option<i64> {
    let text = text.trim_start();
    let digits = text.trim_start_matches(['+', '-']);
    let length = digits.bytes().take_while(u8::is_ascii_digit).count();
    text.get(..text.len() - digits.len() + length)?.parse().ok()
}

pub async fn list(app: &App, uri: &Uri) -> Response {
    let name = query(uri, "name").unwrap_or_default();
    let id = query(uri, "creatorAccountId")
        .as_deref()
        .and_then(parsed_int);
    let kind = query(uri, "type").as_deref().and_then(parsed_int);
    if let (Some(id), Some(kind)) = (id, kind) {
        if !name.is_empty() {
            let Some(store) = &app.settings else {
                return unavailable();
            };
            let key = name.clone();
            match store
                .with_db(move |db| {
                    db.query_row(
                        "SELECT data FROM curated_list WHERE
                creator_id=?1 AND entity_type=?2 AND name=?3",
                        params![id, kind, key],
                        |r| r.get::<_, String>(0),
                    )
                    .optional()
                })
                .await
            {
                Ok(Some(data)) => {
                    return stored_object(
                        &data,
                        &[
                            "ListId",
                            "CreatorAccountId",
                            "Name",
                            "Description",
                            "ImageName",
                            "Type",
                            "ItemIds",
                            "Accessibility",
                            "CreatedAt",
                        ],
                    );
                }
                Ok(None) => {}
                Err(_) => return unavailable(),
            }
        }
    }
    let rows: Vec<Value> =
        serde_json::from_str(include_str!("../compat/catalogue/curated-lists.json"))
            .expect("bundled curated list catalogue");
    let same_name = |row: &&Value| {
        row["Name"]
            .as_str()
            .is_some_and(|n| n.eq_ignore_ascii_case(&name))
    };
    let found = rows
        .iter()
        .filter(same_name)
        .find(|r| r["CreatorAccountId"].as_i64() == id && r["Type"].as_i64() == kind)
        .or_else(|| {
            rows.iter()
                .filter(same_name)
                .find(|r| r["Type"].as_i64() == kind)
        })
        .or_else(|| rows.iter().find(same_name))
        .or_else(|| {
            rows.iter()
                .find(|r| name.is_empty() && r["Type"].as_i64() == kind)
        });
    if let Some(row) = found {
        return json_body(StatusCode::OK, row.to_string());
    }
    let reserved = name.starts_with("__");
    ok(
        json!({"ListId":if reserved {0} else {1},"CreatorAccountId":id.unwrap_or(0),
        "Type":kind.unwrap_or(0),"Name":name,"Description":null,"ItemIds":[],
        "ImageName":if reserved {"DefaultRoomImage.jpg"} else {"DefaultListImage.jpg"},
        "Accessibility":if reserved {1} else {0},
        "CreatedAt":if reserved {"1970-01-01T00:00:00.000Z"} else {"2026-08-24T18:44:52.438Z"}}),
    )
}

pub async fn build(app: &App, uri: &Uri) -> Response {
    let id = query(uri, "cloudBuildId").unwrap_or_default();
    let fail = |status, message| {
        json_body(
            status,
            json!({"success":false,"error":message,"value":null}).to_string(),
        )
    };
    if id.is_empty() || id.starts_with('0') || !id.bytes().all(|b| b.is_ascii_digit()) {
        return fail(StatusCode::BAD_REQUEST, "Invalid build ID.");
    }
    let Some(store) = &app.settings else {
        return unavailable();
    };
    let result = store.with_db(move |db| db.query_row("SELECT started_at,NULLIF(completed_at,''),error,
        unity_asset_id,created_by_account_id FROM studio_cloud_build WHERE cloud_build_id=?1", [&id], |r| {
        let completed:Option<String> = r.get(1)?;
        let error:Option<String> = r.get(2)?;
        let file = (completed.is_some() && error.as_deref().unwrap_or("").is_empty())
            .then(||format!("studio-builds/{id}/server-data.json"));
        Ok(json!({"cloudBuildId":id,"startedAt":r.get::<_,String>(0)?,"completedAt":completed,
            "error":error,"unityAssetId":r.get::<_,String>(3)?,"createdByAccountId":r.get::<_,i64>(4)?,
            "serverDataFileName":file}))
    }).optional()).await;
    match result {
        Ok(Some(row)) => ok(json!({"success":true,"error":null,"value":row})),
        Ok(None) => fail(StatusCode::NOT_FOUND, "No such build."),
        Err(_) => unavailable(),
    }
}
