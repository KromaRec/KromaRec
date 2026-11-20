use crate::{accounts, json_body, not_found, settings::Store};
use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    response::{IntoResponse, Response},
};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(crate) type Fields = BTreeMap<String, String>;
pub(crate) type Reply = (StatusCode, Value);
pub(crate) const DORM: &str = "76d98498-60a1-430c-ab76-b54a29b7a163";
const MAKER: &str = "a75f7547-79eb-47c6-8986-6767abcb4f92";
const MAKER_V2: &str = "5d4e40d8-f289-4295-a6e1-4f907835007d";

pub(crate) struct Room {
    pub id: i64,
    pub owner: Option<i64>,
    pub name: String,
    description: String,
    pub access: i64,
    version: i64,
    pub dorm: bool,
    pub scene: String,
    pub capacity: i64,
    created: String,
    template: Option<String>,
}

impl Room {
    pub fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            owner: row.get("owner")?,
            name: row.get("name")?,
            description: row.get("description")?,
            access: row.get("accessibility")?,
            version: row.get("ugc_version")?,
            dorm: row.get("dorm")?,
            scene: row.get("scene")?,
            capacity: row.get("capacity")?,
            created: row.get("created")?,
            template: row.get("template")?,
        })
    }

    pub fn subroom(&self) -> i64 {
        // Seed data carries its own canonical subroom identifiers.
        self.template_dto()
            .and_then(|value| value["SubRooms"][0]["SubRoomId"].as_i64())
            .unwrap_or(self.id)
    }

    fn template_dto(&self) -> Option<Value> {
        serde_json::from_str(self.template.as_deref()?).ok()
    }

    pub fn manages(&self, db: &Connection, player: i64) -> rusqlite::Result<bool> {
        Ok(self.owner == Some(player) || role(db, self.id, player)? == 30)
    }

    pub fn admits(&self, db: &Connection, player: Option<i64>) -> rusqlite::Result<bool> {
        if self.access != 0 {
            return Ok(true);
        }
        let Some(player) = player else {
            return Ok(false);
        };
        Ok(self.owner == Some(player) || role(db, self.id, player)? > 0)
    }

    fn roles(&self, db: &Connection) -> rusqlite::Result<Value> {
        if let Some(template) = self.template_dto() {
            return Ok(template["Roles"].clone());
        }
        let mut roles = vec![
            json!({"AccountId":self.owner,"Role":255,"InvitedRole":0,"LastChangedByAccountId":null}),
        ];
        let mut rows = db.prepare_cached(
            "SELECT account_id,role,changed_by FROM room_role WHERE room_id=?1 AND role>0",
        )?;
        for row in rows.query_map([self.id], |r| {
            Ok(
                json!({"AccountId":r.get::<_,i64>(0)?,"Role":r.get::<_,i64>(1)?,
                      "InvitedRole":0,"LastChangedByAccountId":r.get::<_,Option<i64>>(2)?}),
            )
        })? {
            roles.push(row?);
        }
        Ok(json!(roles))
    }

    fn dto(&self, db: &Connection) -> rusqlite::Result<Value> {
        if let Some(template) = self.template_dto() {
            return Ok(template);
        }
        Ok(json!({
            "RoomId":self.id,"Name":self.name,"Description":self.description,"ImageName":"",
            "CreatorAccountId":self.owner,"Accessibility":self.access,"State":0,"PublishState":0,
            "WarningMask":0,"CustomWarning":null,"SupportsScreens":true,"SupportsWalkVR":true,
            "SupportsTeleportVR":true,"SupportsVRLow":true,"SupportsQuest2":true,"SupportsMobile":true,
            "SupportsJuniors":true,"SupportsLevelVoting":false,"IsRRO":false,"IsRecRoomApproved":false,
            "IsDorm":self.dorm,"IsPlacePlay":false,"IsDeveloperOwned":false,"IsJuniorCreated":false,
            "ExcludeFromLists":false,"ExcludeFromSearch":false,"CloningAllowed":false,"MinLevel":0,
            "AgeRating":0,"CreatedAt":self.created,"PublishedAt":null,"BecameRRStudioRoomAt":null,
            "Stats":{"CheerCount":0,"FavoriteCount":0,"VisitorCount":0,"VisitCount":0},
            "UgcVersion":self.version,"PersistenceVersion":0,"UgcSubVersion":null,"MinUgcSubVersion":null,
            "MaxPlayers":self.capacity,"MaxPlayerCalculationMode":0,"DisableMicAutoMute":false,
            "DisableRoomComments":false,"EncryptVoiceChat":false,"ToxmodEnabled":false,
            "LoadScreenLocked":false,"AutoLocalizeRoom":false,"LocalizationContext":{},"RankedEntityId":"",
            "Roles":self.roles(db)?,"Tags":[],"Badges":[],
            "SubRooms":[{
                "RoomId":self.id,"SubRoomId":self.id,"Name":"Home","CreatorAccountId":self.owner,
                "UnitySceneId":self.scene,"MaxPlayers":self.capacity,"Accessibility":2,"IsSandbox":!self.dorm,
                "LastModeratedSaveModerationState":0,"ShouldAutoStageSaves":true,
                "SupportsJoinInProgress":false,"CurrentSave":null,"StagedSave":null
            }]
        }))
    }
}

pub(crate) fn by_id(db: &Connection, id: i64) -> rusqlite::Result<Option<Room>> {
    db.prepare_cached("SELECT * FROM room WHERE id=?1")?
        .query_row([id], Room::from_row)
        .optional()
}

pub(crate) fn by_name(db: &Connection, name: &str) -> rusqlite::Result<Option<Room>> {
    db.prepare_cached("SELECT * FROM room WHERE name=?1")?
        .query_row([name], Room::from_row)
        .optional()
}

pub(crate) fn role(db: &Connection, room: i64, player: i64) -> rusqlite::Result<i64> {
    Ok(db
        .prepare_cached("SELECT role FROM room_role WHERE room_id=?1 AND account_id=?2")?
        .query_row(params![room, player], |row| row.get(0))
        .optional()?
        .unwrap_or(0))
}

pub(crate) fn account_exists(db: &Connection, id: i64) -> rusqlite::Result<bool> {
    db.query_row(
        "SELECT EXISTS(SELECT 1 FROM local_account WHERE id=?1)",
        [id],
        |row| row.get(0),
    )
}

pub(crate) fn dorm(db: &Connection, id: i64) -> rusqlite::Result<Room> {
    db.execute(
        "INSERT INTO room(owner,name,description,dorm,scene) SELECT id,
        '@'||username||'''s Dorm','Your private room',1,?2 FROM local_account WHERE id=?1
        AND NOT EXISTS(SELECT 1 FROM room WHERE owner=?1 AND dorm=1)",
        params![id, DORM],
    )?;
    db.query_row(
        "SELECT * FROM room WHERE owner=?1 AND dorm=1",
        [id],
        Room::from_row,
    )
}

pub(crate) fn ok(value: Value) -> Reply {
    (StatusCode::OK, value)
}
pub(crate) fn lower(value: Value, error: Option<&str>) -> Reply {
    ok(json!({"success":error.is_none(),"value":value,"error":error}))
}
fn upper(error: Option<(&str, &str)>) -> Reply {
    ok(
        json!({"Success":error.is_none(),"Value":null,"ErrorId":error.map(|e| e.0),"Error":error.map(|e| e.1)}),
    )
}
pub(crate) fn status(code: StatusCode) -> Reply {
    (code, Value::Null)
}
pub(crate) fn respond(result: rusqlite::Result<Reply>) -> Response {
    match result {
        Ok((StatusCode::NOT_FOUND, Value::Null)) => not_found(),
        Ok((code, Value::Null)) => code.into_response(),
        Ok((code, body)) => json_body(code, body.to_string()),
        Err(error) => {
            eprintln!("room service database error: {error}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 32
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_,".contains(&b))
}

fn page(db: &Connection, query: &Fields, prefix: bool) -> rusqlite::Result<Reply> {
    let terms: Vec<_> = query
        .get("query")
        .map_or("", String::as_str)
        .split(|c: char| c.is_whitespace() || c == '+')
        .filter(|s| !s.is_empty())
        .collect();
    let default_take = if prefix { 10 } else { 30 };
    let take = query
        .get("take")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(default_take)
        .clamp(1, 100);
    let skip = query
        .get("skip")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    let mut stmt =
        db.prepare("SELECT * FROM room WHERE accessibility=1 AND dorm=0 ORDER BY name")?;
    let mut rows = stmt.query([])?;
    let mut total = 0;
    let mut results = Vec::new();
    let needle = query
        .get("query")
        .cloned()
        .unwrap_or_default()
        .to_ascii_lowercase();
    while let Some(row) = rows.next()? {
        let room = Room::from_row(row)?;
        let name = room.name.to_ascii_lowercase();
        let matched = if prefix {
            name.starts_with(&needle)
        } else {
            terms
                .iter()
                .all(|term| match term.to_ascii_lowercase().as_str() {
                    "#community" => room.owner.is_some(),
                    "#rro" => room.owner.is_none(),
                    tag if tag.starts_with('#') => room
                        .template_dto()
                        .and_then(|v| {
                            Some(
                                v["Tags"]
                                    .as_array()?
                                    .iter()
                                    .any(|t| t["Tag"].as_str() == Some(&tag[1..])),
                            )
                        })
                        .unwrap_or(false),
                    word => name.contains(word),
                })
        };
        if matched {
            if total >= skip && results.len() < take {
                results.push(room.dto(db)?);
            }
            total += 1;
        }
    }
    Ok(ok(if prefix {
        json!(results)
    } else {
        json!({"Results":results,"TotalResults":total})
    }))
}

fn create(
    db: &Connection,
    player: i64,
    fields: &Fields,
    source: Option<Room>,
) -> rusqlite::Result<Reply> {
    let clone = source.is_some();
    let name = fields.get("name").map_or("", |name| name.trim());
    let version = if let Some(room) = &source {
        room.version
    } else {
        fields
            .get("ugcVersion")
            .map_or(Some(1), |v| v.parse::<i64>().ok())
            .unwrap_or(0)
    };
    let rejection = if !valid_name(name) {
        Some((
            StatusCode::BAD_REQUEST,
            "A room name of up to 32 letters, numbers, dashes, underscores or commas is required.",
        ))
    } else if !matches!(version, 1 | 2) {
        Some((StatusCode::BAD_REQUEST, "Unsupported UGC version."))
    } else if by_name(db, name)?.is_some() {
        Some((
            StatusCode::CONFLICT,
            "A room with that name already exists.",
        ))
    } else if db.query_row(
        "SELECT count(*) FROM room WHERE owner=?1 AND dorm=0",
        [player],
        |row| row.get::<_, i64>(0),
    )? >= 10
    {
        Some((StatusCode::FORBIDDEN, "You can only have 10 rooms."))
    } else if source.as_ref().is_some_and(|r| {
        r.template_dto()
            .is_none_or(|value| value["CloningAllowed"] != true)
    }) {
        Some((StatusCode::FORBIDDEN, "You can't clone this room!"))
    } else {
        None
    };
    if let Some((code, error)) = rejection {
        return Ok(if clone {
            lower(Value::Null, Some(error))
        } else {
            (code, json!({"error":error}))
        });
    }
    let (scene, capacity) = source.map_or_else(
        || (if version == 2 { MAKER_V2 } else { MAKER }.to_owned(), 4),
        |r| (r.scene, r.capacity),
    );
    let id = db.query_row("INSERT INTO room(owner,name,ugc_version,scene,capacity) VALUES(?1,?2,?3,?4,?5) RETURNING id",
        params![player,name,version,scene,capacity], |row| row.get(0))?;
    let value = by_id(db, id)?
        .ok_or(rusqlite::Error::QueryReturnedNoRows)?
        .dto(db)?;
    Ok(if clone { lower(value, None) } else { ok(value) })
}

pub(crate) fn parts(uri: &axum::http::Uri) -> (Vec<String>, Fields) {
    let path = uri
        .path()
        .trim_matches('/')
        .split('/')
        .map(|part| {
            percent_encoding::percent_decode_str(part)
                .decode_utf8_lossy()
                .into_owned()
        })
        .collect();
    let fields = form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
        .into_owned()
        .collect();
    (path, fields)
}

pub(crate) async fn fields(request: Request<Body>) -> Result<Fields, StatusCode> {
    Ok(field_pairs(request).await?.into_iter().collect())
}

pub(crate) async fn field_pairs(
    request: Request<Body>,
) -> Result<Vec<(String, String)>, StatusCode> {
    if request
        .headers()
        .get(axum::http::header::CONTENT_LENGTH)
        .is_some_and(|v| v == "0")
        || request.body().size_hint().exact() == Some(0)
    {
        return Ok(Vec::new());
    }
    accounts::form_pairs(request).await
}

use axum::body::HttpBody;

pub async fn serve(
    store: Option<&Store>,
    hub: &crate::notify::Hub,
    request: Request<Body>,
    method: Method,
) -> Response {
    let (path, query) = parts(request.uri());
    let Some(store) = store else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    let player = store.authenticate(request.headers()).await;
    if path == ["rooms", "bulk"] && matches!(method, Method::GET | Method::POST) {
        return bulk(store, player, method, request).await;
    }

    let body = if matches!(method, Method::POST | Method::PUT) {
        if player.is_none() {
            return StatusCode::UNAUTHORIZED.into_response();
        }
        match fields(request).await {
            Ok(fields) => fields,
            Err(code) => return code.into_response(),
        }
    } else {
        Fields::new()
    };
    let result = store
        .with_db(move |db| {
            let tx = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
            let reply = route(&tx, &path, &method, player, &query, &body)?;
            let update = if method == Method::PUT
                && matches!(
                    path.get(2).map(String::as_str),
                    Some("name" | "description" | "accessibility" | "creator" | "roles")
                )
                && (reply.1["Success"] == true || reply.1["success"] == true)
                && let Some(id) = path.get(1).and_then(|id| id.parse::<i64>().ok())
                && let Some(room) = by_id(&tx, id)?
            {
                let mut recipients = std::collections::BTreeSet::new();
                recipients.extend(player);
                recipients.extend(room.owner);
                let mut stmt = tx.prepare_cached(
                    "SELECT l.player FROM room_location l JOIN room_instance i ON i.id=l.instance
                WHERE i.room_id=?1 AND l.expires>unixepoch() LIMIT 128",
                )?;
                for row in stmt.query_map([id], |r| r.get::<_, i64>(0))? {
                    recipients.insert(row?);
                }
                if let Some(target) = path.get(3).and_then(|id| id.parse::<i64>().ok()) {
                    recipients.insert(target);
                }
                let invite = (path.get(4).map(String::as_str) == Some("invite"))
                    .then(|| path[3].parse::<i64>().ok())
                    .flatten();
                let invite = invite
                    .map(|target| {
                        let (message_id,sent) = tx.query_row(
                        "INSERT INTO social_inbox(sender,recipient,kind,data,room_id) VALUES(?1,?2,62,'30',?3)
                         RETURNING id,sent", params![player,target,id],
                        |r| Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?)))?;
                    Ok::<_,rusqlite::Error>((target,json!({"Id":message_id,"FromPlayerId":player,"ToPlayerId":target,
                        "RoomId":id,"Type":62,"Data":"30","SentTime":sent})))
                    }).transpose()?;
                Some((recipients, room.dto(&tx)?, invite))
            } else { None };
            tx.commit()?;
            Ok((reply, update))
        }).await;
    match result {
        Ok((reply, update)) => {
            if let Some((recipients, room, invite)) = update {
                for player in recipients {
                    hub.send(player, "RoomUpdate", room.clone());
                }
                if let Some((player, message)) = invite {
                    hub.send(player, "2", message);
                }
            }
            respond(Ok(reply))
        }
        Err(error) => respond(Err(error)),
    }
}

fn route(
    db: &Connection,
    path: &[String],
    method: &Method,
    player: Option<i64>,
    query: &Fields,
    fields: &Fields,
) -> rusqlite::Result<Reply> {
    let path: Vec<_> = path.iter().map(String::as_str).collect();
    let private_read = matches!(
        path.as_slice(),
        ["dormroom", "me"]
            | ["photon_access_token"]
            | ["roomserver", "rooms", "createdby", "me"]
            | ["rooms", "ownedby" | "createdby" | "contributedby", "me"]
    );
    if method == Method::GET && !private_read {
        return read(db, &path, player, query);
    }
    let Some(account) = player else {
        return Ok(status(StatusCode::UNAUTHORIZED));
    };
    if !account_exists(db, account)? {
        return Ok(status(StatusCode::UNAUTHORIZED));
    }
    if method == Method::GET {
        match path.as_slice() {
            ["photon_access_token"] => return crate::sessions::permissions(db, account),
            ["dormroom", "me"] => return Ok(ok(json!(dorm(db, account)?.id))),
            _ => {}
        }
        let filter = match path.as_slice() {
            ["rooms", "ownedby", "me"] => {
                "dorm=0 AND (owner=?1 OR id IN (SELECT room_id FROM room_role WHERE account_id=?1 AND role=30))"
            }
            ["rooms", "contributedby", "me"] => {
                "owner=?1 OR id IN (SELECT room_id FROM room_role WHERE account_id=?1 AND role>0)"
            }
            _ => "owner=?1",
        };
        let mut rows =
            db.prepare_cached(&format!("SELECT * FROM room WHERE {filter} ORDER BY id"))?;
        let mut list = Vec::new();
        for room in rows.query_map([account], Room::from_row)? {
            list.push(room?.dto(db)?);
        }
        return Ok(ok(json!(list)));
    }
    if *method == Method::POST && path == ["rooms", "create"] {
        return create(db, account, fields, None);
    }
    let ["rooms", id, rest @ ..] = path.as_slice() else {
        return Ok(status(StatusCode::NOT_FOUND));
    };
    let Ok(id) = id.parse::<i64>() else {
        return Ok(status(StatusCode::NOT_FOUND));
    };
    let missing = ("Rooms.DoesntExist", "This room does not exist!");
    let denied = ("Rooms.NotOwner", "You are not the owner of this room!");
    let reject = |error| lower(Value::Null, Some(error));
    let old_envelope = (*method == Method::DELETE && rest.is_empty())
        || (*method == Method::PUT && matches!(rest, ["name" | "description"]));
    let Some(room) = by_id(db, id)? else {
        return Ok(if old_envelope {
            upper(Some(missing))
        } else {
            reject(missing.1)
        });
    };
    if *method == Method::POST && rest == ["clone"] {
        let mut fields = fields.clone();
        if let Some(name) = query.get("name") {
            fields.entry("name".into()).or_insert_with(|| name.clone());
        }
        return create(db, account, &fields, Some(room));
    }
    if old_envelope {
        if room.owner != Some(account) {
            return Ok(upper(Some(denied)));
        }
        match rest {
            [] => {
                db.execute("DELETE FROM room WHERE id=?1", [id])?;
            }
            ["description"] => {
                db.execute(
                    "UPDATE room SET description=?2 WHERE id=?1",
                    params![id, fields.get("description").map_or("", String::as_str)],
                )?;
            }
            ["name"] => {
                let name = fields.get("name").map_or("", |v| v.trim());
                if !valid_name(name) {
                    return Ok(upper(Some((
                        "Rooms.InvalidName",
                        "You must provide a valid room name!",
                    ))));
                }
                if by_name(db, name)?.is_some_and(|r| r.id != id) {
                    return Ok(upper(Some((
                        "Rooms.AlreadyExists",
                        "A room with that name already exists!",
                    ))));
                }
                db.execute("UPDATE room SET name=?2 WHERE id=?1", params![id, name])?;
            }
            _ => unreachable!(),
        }
        return Ok(upper(None));
    }
    if *method != Method::PUT {
        return Ok(status(StatusCode::NOT_FOUND));
    }
    match rest {
        ["accessibility"] => {
            if !room.manages(db, account)? {
                return Ok(status(StatusCode::FORBIDDEN));
            }
            let Some(access) = number(fields, "accessibility").filter(|v| (0..=2).contains(v))
            else {
                return Ok(reject("You must provide a valid accessibility!"));
            };
            if room.dorm && access == 1 {
                return Ok(reject("A dorm cannot be published!"));
            }
            db.execute(
                "UPDATE room SET accessibility=?2 WHERE id=?1",
                params![id, access],
            )?;
            if access == 0 && room.access != 0 {
                db.execute("DELETE FROM room_instance WHERE room_id=?1", [id])?;
            }
        }
        ["creator"] => {
            if room.owner != Some(account) {
                return Ok(status(StatusCode::FORBIDDEN));
            }
            if room.dorm {
                return Ok(reject("A dorm cannot be given away!"));
            }
            let Some(target) = number(fields, "accountId") else {
                return Ok(reject("You must provide an account!"));
            };
            if target == account {
                return Ok(reject("You already own this room!"));
            }
            if !account_exists(db, target)? {
                return Ok(reject("That player does not exist!"));
            }
            db.execute("UPDATE room SET owner=?2 WHERE id=?1", params![id, target])?;
            db.execute(
                "DELETE FROM room_role WHERE room_id=?1 AND account_id=?2",
                params![id, target],
            )?;
            set_role(db, id, account, 30, target)?;
        }
        ["roles", target, "invite"] => {
            if room.owner != Some(account) {
                return Ok(status(StatusCode::FORBIDDEN));
            }
            let Ok(target) = target.parse::<i64>() else {
                return Ok(status(StatusCode::NOT_FOUND));
            };
            if target == account
                || !account_exists(db, target)?
                || number(fields, "role") != Some(30)
            {
                return Ok(reject("You must provide a valid co-owner invite!"));
            }
            db.execute("INSERT INTO room_role(room_id,account_id,role,changed_by,invited_role)
                VALUES(?1,?2,0,?3,30) ON CONFLICT(room_id,account_id) DO UPDATE SET invited_role=30",
                params![id,target,account])?;
            return Ok(ok(json!({"success":true})));
        }
        ["roles", target] => {
            let Ok(target) = target.parse::<i64>() else {
                return Ok(status(StatusCode::NOT_FOUND));
            };
            let tier = match (fields.contains_key("role"), number(fields, "role")) {
                (_, Some(v)) => v,
                (false, None) => 0,
                _ => return Ok(reject("You must provide a valid role!")),
            };
            if target == account {
                let pending = db
                    .query_row(
                        "SELECT invited_role FROM room_role WHERE room_id=?1 AND account_id=?2",
                        params![id, target],
                        |r| r.get::<_, i64>(0),
                    )
                    .optional()?
                    .unwrap_or(0);
                if pending == 0 || !fields.contains_key("role") || (tier != 0 && tier != pending) {
                    return Ok(reject("You have no such invite to this room!"));
                }
                if tier == 0 {
                    remove_role(db, id, target)?;
                } else {
                    db.execute("UPDATE room_role SET role=invited_role,invited_role=0 WHERE room_id=?1 AND account_id=?2",params![id,target])?;
                }
            } else {
                if !room.manages(db, account)? {
                    return Ok(status(StatusCode::FORBIDDEN));
                }
                if room.owner == Some(target)
                    || !account_exists(db, target)?
                    || (role(db, id, target)? == 30 && room.owner != Some(account))
                {
                    return Ok(reject("You cannot change that role!"));
                }
                match tier {
                    0 => remove_role(db, id, target)?,
                    10 | 20 => set_role(db, id, target, tier, account)?,
                    _ => return Ok(reject("Co-ownership must be invited, not granted!")),
                }
            }
        }
        _ => return Ok(status(StatusCode::NOT_FOUND)),
    }
    Ok(lower(
        by_id(db, id)?
            .ok_or(rusqlite::Error::QueryReturnedNoRows)?
            .dto(db)?,
        None,
    ))
}

fn number(fields: &Fields, key: &str) -> Option<i64> {
    fields.get(key)?.trim().parse().ok()
}

fn set_role(
    db: &Connection,
    room: i64,
    player: i64,
    role: i64,
    changed_by: i64,
) -> rusqlite::Result<()> {
    db.execute("INSERT INTO room_role(room_id,account_id,role,changed_by) VALUES(?1,?2,?3,?4)
        ON CONFLICT(room_id,account_id) DO UPDATE SET role=excluded.role,changed_by=excluded.changed_by",
        params![room,player,role,changed_by])?;
    Ok(())
}

fn remove_role(db: &Connection, room: i64, player: i64) -> rusqlite::Result<()> {
    db.execute(
        "DELETE FROM room_role WHERE room_id=?1 AND account_id=?2",
        params![room, player],
    )?;
    db.execute(
        "DELETE FROM room_location WHERE player=?2 AND instance IN
        (SELECT room_instance.id FROM room_instance JOIN room ON room.id=room_instance.room_id
         WHERE room.id=?1 AND room.accessibility=0)",
        params![room, player],
    )?;
    Ok(())
}

async fn bulk(
    store: &Store,
    player: Option<i64>,
    method: Method,
    request: Request<Body>,
) -> Response {
    let fields: Vec<_> = if method == Method::POST {
        if request.body().size_hint().exact() == Some(0) {
            Vec::new()
        } else {
            match accounts::form_pairs(request).await {
                Ok(fields) => fields,
                Err(status) => return status.into_response(),
            }
        }
    } else {
        form_urlencoded::parse(request.uri().query().unwrap_or_default().as_bytes())
            .into_owned()
            .collect()
    };
    let have_ids = fields.iter().any(|(k, _)| k.eq_ignore_ascii_case("id"));
    let ids: Vec<i64> = fields
        .iter()
        .filter(|(k, _)| k.eq_ignore_ascii_case("id"))
        .flat_map(|(_, v)| v.split(','))
        .filter_map(|v| v.trim().parse().ok())
        .take(101)
        .collect();
    let name = fields
        .iter()
        .find(|(k, v)| k == "name" && !v.is_empty())
        .map(|(_, v)| v.clone());
    let public = fields.iter().any(|(k, v)| {
        k.eq_ignore_ascii_case("excludePrivateRooms") && v.eq_ignore_ascii_case("true")
    });
    if ids.len() > 100 {
        return json_body(
            StatusCode::BAD_REQUEST,
            json!("Too many room ids").to_string(),
        );
    }
    if method == Method::GET && !have_ids && name.is_none() {
        return json_body(
            StatusCode::BAD_REQUEST,
            json!("Either 'id' or 'name' query parameter is required").to_string(),
        );
    }
    respond(
        store
            .with_db(move |db| {
                let mut found = Vec::new();
                if method == Method::GET && !have_ids {
                    if let Some(room) = by_name(db, name.as_deref().unwrap_or_default())?
                        && (!public || room.access == 1)
                        && can_read(db, &room, player)?
                    {
                        found.push(room.dto(db)?);
                    }
                } else {
                    let mut seen = std::collections::HashSet::new();
                    for id in ids {
                        if seen.insert(id)
                            && let Some(room) = by_id(db, id)?
                            && (!public || room.access == 1)
                            && can_read(db, &room, player)?
                        {
                            found.push(room.dto(db)?);
                        }
                    }
                }
                Ok(ok(json!(found)))
            })
            .await,
    )
}

fn can_read(db: &Connection, room: &Room, player: Option<i64>) -> rusqlite::Result<bool> {
    if room.admits(db, player)? {
        return Ok(true);
    }
    let Some(player) = player else {
        return Ok(false);
    };
    // After a private invite has been redeemed, the guest needs this DTO to load the scene.
    db.query_row(
        "SELECT EXISTS(SELECT 1 FROM room_location l JOIN room_instance i ON i.id=l.instance
        WHERE i.room_id=?1 AND l.player=?2 AND l.expires>unixepoch())",
        params![room.id, player],
        |r| r.get(0),
    )
}

fn read(
    db: &Connection,
    path: &[&str],
    player: Option<i64>,
    query: &Fields,
) -> rusqlite::Result<Reply> {
    match path {
        ["rooms", "search"] => page(db, query, false),
        ["rooms", "search", "name"] => page(db, query, true),
        ["rooms", "base"] => {
            let mut stmt = db.prepare_cached("SELECT * FROM room WHERE template IS NOT NULL AND id NOT IN (2,13) ORDER BY id LIMIT ?1 OFFSET ?2")?;
            let take = number(query, "take").unwrap_or(100).clamp(1, 100);
            let skip = number(query, "skip").unwrap_or(0).max(0);
            let mut list = Vec::new();
            for row in stmt.query_map(params![take, skip], Room::from_row)? {
                list.push(row?.dto(db)?);
            }
            Ok(ok(json!(list)))
        }
        ["rooms"] => {
            let room = if let Some(ids) = query.get("id").filter(|v| !v.is_empty()) {
                match ids.split(',').find_map(|v| v.trim().parse::<i64>().ok()) {
                    Some(id) => by_id(db, id)?,
                    None => None,
                }
            } else if let Some(name) = query.get("name").filter(|v| !v.is_empty()) {
                by_name(db, name)?
            } else {
                return Ok((
                    StatusCode::BAD_REQUEST,
                    json!("Either 'id' or 'name' query parameter is required"),
                ));
            };
            if let Some(room) = room
                && can_read(db, &room, player)?
            {
                return Ok(ok(room.dto(db)?));
            }
            Ok(ok(json!({})))
        }
        ["rooms", id] | ["rooms", id, "roles"] => {
            let room = match id.parse::<i64>() {
                Ok(id) => by_id(db, id)?,
                Err(_) => None,
            };
            if let Some(room) = room
                && can_read(db, &room, player)?
            {
                return Ok(ok(if path.len() == 3 {
                    room.roles(db)?
                } else {
                    room.dto(db)?
                }));
            }
            Ok(if path.len() == 3 {
                ok(json!([]))
            } else {
                status(StatusCode::NOT_FOUND)
            })
        }
        _ => Ok(status(StatusCode::NOT_FOUND)),
    }
}
