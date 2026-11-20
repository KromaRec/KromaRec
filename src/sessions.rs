use crate::{
    accounts, auth, notify, presence,
    rooms::{self, Fields, Reply},
    settings::Store,
};
use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    response::{IntoResponse, Response},
};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};
use std::env;

const TTL: i64 = 900;
const EMPTY_GRACE: i64 = 120;
const MAX_PARTY_INVITEES: usize = 16;
const NO_ROOM: i64 = 20;

pub struct Photon {
    pub region: String,
    pub(crate) realtime: String,
    voice: String,
    chat: String,
}

impl Photon {
    pub fn from_env() -> Self {
        let value = |key| env::var(key).unwrap_or_default().trim().to_owned();
        let region = value("KROMAREC_PHOTON_REGION");
        Self {
            region: if region.is_empty() {
                "us".into()
            } else {
                region
            },
            realtime: value("KROMAREC_PHOTON_REALTIME_APP_ID"),
            voice: value("KROMAREC_PHOTON_VOICE_APP_ID"),
            chat: value("KROMAREC_PHOTON_CHAT_APP_ID"),
        }
    }
}

pub(crate) fn uuid() -> rusqlite::Result<String> {
    let mut bytes = accounts::random::<16>()?;
    bytes[6] = (bytes[6] & 15) | 64;
    bytes[8] = (bytes[8] & 63) | 128;
    let hex: String = bytes.iter().map(|n| format!("{n:02x}")).collect();
    Ok(format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    ))
}

struct Instance {
    id: i64,
    room: i64,
    photon_room: String,
    private: bool,
    in_progress: bool,
    created: String,
}

impl Instance {
    fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            room: row.get("room_id")?,
            photon_room: row.get("photon_room")?,
            private: row.get("private")?,
            in_progress: row.get("in_progress")?,
            created: row.get("created")?,
        })
    }

    fn dto(&self, room: &rooms::Room, region: &str, full: bool) -> Value {
        let name = if room.dorm {
            room.name.clone()
        } else {
            format!("^{}", room.name)
        };
        json!({
            "roomInstanceId":self.id,"roomId":room.id,"subRoomId":room.subroom(),
            "roomInstanceType":if room.dorm {2} else {0}, "location":room.scene,
            "dataBlob":"","eventId":0,"clubId":0,"roomCode":"","photonRegion":region,
            "photonRegionId":region,"photonRoomId":self.photon_room,"name":name,
            "maxCapacity":room.capacity,"isFull":full,"isPrivate":self.private,"isInProgress":self.in_progress,
            "EncryptVoiceChat":false
        })
    }
}

fn by_id(db: &Connection, id: i64) -> rusqlite::Result<Option<Instance>> {
    db.prepare_cached("SELECT * FROM room_instance WHERE id=?1")?
        .query_row([id], Instance::from_row)
        .optional()
}

fn current(db: &Connection, player: i64) -> rusqlite::Result<Option<Instance>> {
    db.prepare_cached(
        "SELECT i.* FROM room_location l JOIN room_instance i ON i.id=l.instance
        WHERE l.player=?1 AND l.expires>unixepoch()",
    )?
    .query_row([player], Instance::from_row)
    .optional()
}

pub(crate) fn start_orientation(db: &Connection, player: i64) -> rusqlite::Result<()> {
    db.execute(
        "DELETE FROM onboarding_presence WHERE expires<=unixepoch()",
        [],
    )?;
    db.execute(
        "INSERT INTO onboarding_presence(player,expires) VALUES(?1,unixepoch()+?2)",
        params![player, TTL],
    )?;
    Ok(())
}

pub(crate) fn in_orientation(db: &Connection, player: i64) -> rusqlite::Result<bool> {
    db.prepare_cached(
        "SELECT EXISTS(SELECT 1 FROM onboarding_presence WHERE player=?1 AND expires>unixepoch())",
    )?
    .query_row([player], |r| r.get(0))
}

fn orientation(db: &Connection, player: i64, region: &str) -> rusqlite::Result<Option<Value>> {
    if !in_orientation(db, player)? {
        return Ok(None);
    }
    let room = rooms::by_id(db, 13)?.ok_or(rusqlite::Error::QueryReturnedNoRows)?;
    let instance = Instance {
        id: -2,
        room: 13,
        photon_room: "rec.13".into(),
        private: false,
        in_progress: false,
        created: String::new(),
    };
    Ok(Some(instance.dto(&room, region, false)))
}

pub(crate) fn photon_room(db: &Connection, player: i64) -> rusqlite::Result<Option<String>> {
    db.prepare_cached(
        "SELECT i.photon_room FROM room_location l
        JOIN room_instance i ON i.id=l.instance
        WHERE l.player=?1 AND l.expires>unixepoch()
        UNION ALL
        SELECT 'rec.13' FROM onboarding_presence WHERE player=?1 AND expires>unixepoch()
        LIMIT 1",
    )?
    .query_row([player], |row| row.get(0))
    .optional()
}

pub(crate) fn current_dto(db: &Connection, player: i64, region: &str) -> rusqlite::Result<Value> {
    let Some(instance) = current(db, player)? else {
        return Ok(orientation(db, player, region)?.unwrap_or(Value::Null));
    };
    let room = rooms::by_id(db, instance.room)?.ok_or(rusqlite::Error::QueryReturnedNoRows)?;
    Ok(instance.dto(&room, region, count(db, instance.id)? >= room.capacity))
}

pub(crate) fn leave(db: &Connection, player: i64) -> rusqlite::Result<()> {
    db.execute("UPDATE room_instance SET touched=unixepoch() WHERE id=(SELECT instance FROM room_location WHERE player=?1)",[player])?;
    db.execute("DELETE FROM room_location WHERE player=?1", [player])?;
    Ok(())
}

pub(crate) fn refresh(db: &Connection, player: i64) -> rusqlite::Result<()> {
    db.execute(
        "UPDATE onboarding_presence SET expires=unixepoch()+?2
        WHERE player=?1 AND expires>unixepoch() AND expires<=unixepoch()+300",
        params![player, TTL],
    )?;
    db.execute(
        "UPDATE room_location SET expires=unixepoch()+?2 WHERE player=?1 AND expires>unixepoch() AND expires<=unixepoch()+300",
        params![player, TTL],
    )?;
    Ok(())
}

fn sweep(db: &Connection) -> rusqlite::Result<()> {
    // Preserve a short grace window after the last known expiry as well as after explicit departure.
    db.execute(
        "UPDATE room_instance SET touched=unixepoch()
        WHERE touched<unixepoch()-?1 AND id IN
        (SELECT instance FROM room_location WHERE expires BETWEEN unixepoch()-?1 AND unixepoch())",
        [EMPTY_GRACE],
    )?;
    db.execute("DELETE FROM room_location WHERE expires<=unixepoch()", [])?;
    db.execute("DELETE FROM room_invite WHERE expires<=unixepoch()", [])?;
    db.execute(
        "DELETE FROM room_instance WHERE touched<unixepoch()-?1 AND NOT EXISTS
        (SELECT 1 FROM room_location WHERE instance=room_instance.id)",
        [EMPTY_GRACE],
    )?;
    Ok(())
}

fn count(db: &Connection, instance: i64) -> rusqlite::Result<i64> {
    db.query_row(
        "SELECT count(*) FROM room_location WHERE instance=?1 AND expires>unixepoch()",
        [instance],
        |r| r.get(0),
    )
}

fn allocate(
    db: &Connection,
    room: &rooms::Room,
    player: i64,
    private_requested: bool,
) -> rusqlite::Result<Instance> {
    let shared = room.dorm || room.access == 0;
    let private = shared || private_requested;
    let instance = db.prepare_cached("SELECT i.* FROM room_instance i WHERE room_id=?1
        AND private=?2 AND shared_private=?3 AND (?2=0 OR ?3=1 OR host=?4)
        AND ((in_progress=0 AND matchmaking_policy<>2) OR id=(SELECT instance FROM room_location WHERE player=?4))
        AND (SELECT count(*) FROM room_location l WHERE l.instance=i.id AND l.player<>?4 AND l.expires>unixepoch())<?5
        ORDER BY (id=(SELECT instance FROM room_location WHERE player=?4)) DESC,matchmaking_policy,id LIMIT 1")?
        .query_row(params![room.id,private,shared,player,room.capacity],Instance::from_row).optional()?;
    if let Some(instance) = instance {
        return Ok(instance);
    }
    db.query_row(
        "INSERT INTO room_instance(room_id,host,photon_room,private,shared_private,touched)
        VALUES(?1,?2,?3,?4,?5,unixepoch()) RETURNING *",
        params![room.id, player, uuid()?, private, shared],
        Instance::from_row,
    )
}

fn mutually_visible(
    db: &Connection,
    player: i64,
    peer: i64,
    friends: bool,
) -> rusqlite::Result<bool> {
    db.query_row("SELECT ?1<>?2 AND (?3=0 OR
         (SELECT count(*) FROM relationship WHERE kind=3 AND ((owner=?1 AND peer=?2) OR (owner=?2 AND peer=?1)))=2)
         AND NOT EXISTS(SELECT 1 FROM relationship WHERE ignored<>0 AND ((owner=?1 AND peer=?2) OR (owner=?2 AND peer=?1)))",
         params![player,peer,friends], |r|r.get(0))
}

fn invited(db: &Connection, player: i64, instance: i64) -> rusqlite::Result<bool> {
    db.query_row(
        "SELECT EXISTS(SELECT 1 FROM room_invite i JOIN room_location l ON l.player=i.sender
        WHERE i.recipient=?1 AND i.instance=?2 AND i.expires>unixepoch() AND l.instance=i.instance
        AND l.expires>unixepoch() AND NOT EXISTS(SELECT 1 FROM relationship WHERE ignored<>0
        AND ((owner=?1 AND peer=i.sender) OR (owner=i.sender AND peer=?1))))",
        params![player, instance],
        |r| r.get(0),
    )
}

fn entered(db: &Connection, player: i64, instance: i64) -> rusqlite::Result<()> {
    db.execute("DELETE FROM onboarding_presence WHERE player=?1", [player])?;
    if current(db, player)?.is_none_or(|active| active.id != instance) {
        leave(db, player)?;
    }
    db.execute(
        "INSERT INTO room_location(player,instance,expires) VALUES(?1,?2,unixepoch()+?3)
        ON CONFLICT(player) DO UPDATE SET instance=excluded.instance,expires=excluded.expires",
        params![player, instance, TTL],
    )?;
    db.execute("UPDATE lobby_presence SET expires=unixepoch()+?2 WHERE account_id=?1 AND expires>unixepoch()",params![player,TTL])?;
    Ok(())
}

fn field<'a>(fields: &'a Fields, name: &str) -> Option<&'a str> {
    fields
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
        .filter(|v| !v.is_empty())
}

fn numeric(value: &str) -> i64 {
    value.parse().unwrap_or(0)
}

fn result(code: i64, instance: Value, correlation: &str) -> Reply {
    rooms::ok(
        json!({"errorCode":code,"result":code,"roomInstance":instance,"correlationId":correlation}),
    )
}

fn matchmake(
    db: &Connection,
    path: &[&str],
    player: i64,
    region: &str,
    query: &Fields,
    fields: &Fields,
) -> rusqlite::Result<Reply> {
    let correlation = field(fields, "CorrelationId")
        .or_else(|| field(query, "CorrelationId"))
        .unwrap_or("00000000-0000-0000-0000-000000000000");
    let fail = |code| Ok(result(code, Value::Null, correlation));
    if crate::photon_gate::banned(db, player)? {
        return fail(55);
    }
    let private = field(fields, "JoinMode") == Some("2");
    if path == ["matchmake", "none"]
        && let Some(instance) = orientation(db, player, region)?
    {
        refresh(db, player)?;
        return Ok(result(0, instance, correlation));
    }
    let pair = match path {
        ["matchmake", "none"] if current(db, player)?.is_some() => {
            let instance = current(db, player)?.ok_or(rusqlite::Error::QueryReturnedNoRows)?;
            let room =
                rooms::by_id(db, instance.room)?.ok_or(rusqlite::Error::QueryReturnedNoRows)?;
            (room, instance)
        }
        ["matchmake", "none" | "dorm"] => {
            let room = rooms::dorm(db, player)?;
            let instance = allocate(db, &room, player, true)?;
            (room, instance)
        }
        ["matchmake", "room", name] | ["matchmake", "room", name, _] => {
            let room = match name.parse::<i64>() {
                Ok(id) => rooms::by_id(db, id)?,
                Err(_) => rooms::by_name(db, name.trim_start_matches('^'))?,
            };
            let Some(room) = room else {
                return fail(NO_ROOM);
            };
            if !room.admits(db, Some(player))? {
                return fail(25);
            }
            let instance = allocate(db, &room, player, private)?;
            (room, instance)
        }
        ["matchmake", "player", target] => {
            let target = numeric(target);
            let Some(instance) = current(db, target)? else {
                return fail(NO_ROOM);
            };
            let room =
                rooms::by_id(db, instance.room)?.ok_or(rusqlite::Error::QueryReturnedNoRows)?;
            let has_invite = invited(db, player, instance.id)?;
            if !mutually_visible(db, player, target, false)?
                || (!has_invite
                    && (!mutually_visible(db, player, target, true)?
                        || !room.admits(db, Some(player))?
                        || (instance.private && !room.manages(db, player)?)))
            {
                return fail(NO_ROOM);
            }
            if current(db, player)?.is_none_or(|i| i.id != instance.id)
                && count(db, instance.id)? >= room.capacity
            {
                return fail(3);
            }
            (room, instance)
        }
        ["matchmake", "instance", target] => {
            let Some(instance) = by_id(db, numeric(target))? else {
                return fail(NO_ROOM);
            };
            let room =
                rooms::by_id(db, instance.room)?.ok_or(rusqlite::Error::QueryReturnedNoRows)?;
            if !room.manages(db, player)? && !invited(db, player, instance.id)? {
                return fail(NO_ROOM);
            }
            if current(db, player)?.is_none_or(|existing| existing.id != instance.id)
                && count(db, instance.id)? >= room.capacity
            {
                return fail(3);
            }
            (room, instance)
        }
        _ => return Ok(rooms::status(StatusCode::NOT_FOUND)),
    };
    let (room, instance) = pair;
    entered(db, player, instance.id)?;
    Ok(result(
        0,
        instance.dto(&room, region, count(db, instance.id)? >= room.capacity),
        correlation,
    ))
}

fn connection_info(
    db: &Connection,
    player: i64,
    photon: &Photon,
    secret: &[u8],
) -> rusqlite::Result<Reply> {
    let room = photon_room(db, player)?.unwrap_or_default();
    let (access_generation, photon_generation, expires): (i64, i64, i64) = db.prepare_cached(
        "SELECT access_generation, photon_generation, unixepoch()+900 FROM local_account WHERE id=?1",
    )?.query_row([player], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
    let token = auth::sign_claims(
        secret,
        &json!({
            "sub":player.to_string(),"rn.platid":"","rn.plat":"0","rn.deviceclass":"0","rn.env":"prod","kromarec_use":"photon-realtime",
            "exp":expires,"aud":photon.realtime,
            "kromarec_access_generation":access_generation,"kromarec_photon_generation":photon_generation
        }),
    )?;
    Ok(rooms::ok(json!({"success":true,"error":null,"value":{
        "photonAuthToken":token,"photonRealtimeAppId":photon.realtime,"photonVoiceAppId":photon.voice,
        "photonChatAppId":photon.chat,"photonRegion":photon.region,"photonRoomId":room,
        "voiceConnectionInfo":"","voiceServerId":"","experiments":{
            "networkTransformSyncInterval":10.0,"shouldUseUnreliableOnChange":false,
            "shouldAvoidDiscontinuityRPCs":true,"shouldAvoidRedundantDiscontinuity":false,
            "r2RuntimeStaticBaking":true,"r2AutoEmbodiment":true,"r2RuntimeStaticBakingMinShapeThreshold":1,
            "r2UseCheapReplicas":true,"shouldUseGameServerNetworking":false
        }
    }})))
}

fn invite(
    db: &Connection,
    player: i64,
    fields: &Fields,
) -> rusqlite::Result<(Reply, Option<Value>)> {
    let recipient = field(fields, "playerId").map_or(0, numeric);
    let instance = field(fields, "roomInstanceId").map_or(0, numeric);
    if recipient <= 0 {
        return Ok((rooms::status(StatusCode::BAD_REQUEST), None));
    }
    let Some(active) = current(db, player)?.filter(|i| i.id == instance) else {
        return Ok((rooms::status(StatusCode::FORBIDDEN), None));
    };
    if !mutually_visible(db, player, recipient, false)? || !rooms::account_exists(db, recipient)? {
        return Ok((rooms::status(StatusCode::FORBIDDEN), None));
    }
    let (invite_id, message) = record_invite(db, player, recipient, &active)?;
    Ok((
        rooms::ok(
            json!({"RoomInviteId":invite_id,"FromPlayerId":player,"ToPlayerId":recipient,"RoomId":active.room}),
        ),
        Some(message),
    ))
}

fn record_invite(
    db: &Connection,
    player: i64,
    recipient: i64,
    active: &Instance,
) -> rusqlite::Result<(i64, Value)> {
    let invite_id: i64 = db.query_row(
        "INSERT INTO room_invite(sender,recipient,instance,expires)
        VALUES(?1,?2,?3,unixepoch()+300)
        RETURNING id",
        params![player, recipient, active.id],
        |r| r.get(0),
    )?;
    let sent: String = db.query_row("SELECT strftime('%Y-%m-%dT%H:%M:%fZ','now')", [], |r| {
        r.get(0)
    })?;
    Ok((
        invite_id,
        json!({"Id":invite_id,"FromPlayerId":player,"ToPlayerId":recipient,
        "RoomId":active.room,"Type":0,"Data":active.id.to_string(),"SentTime":sent}),
    ))
}

fn invite_party(db: &Connection, player: i64, recipients: &[i64]) -> rusqlite::Result<Vec<Value>> {
    let Some(active) = current(db, player)? else {
        return Ok(Vec::new());
    };
    let mut messages = Vec::new();
    for &recipient in recipients {
        if mutually_visible(db, player, recipient, false)?
            && rooms::account_exists(db, recipient)?
            && !crate::photon_gate::banned(db, recipient)?
            && current(db, recipient)?.is_none_or(|i| i.id != active.id)
        {
            let pending: bool = db
                .prepare_cached(
                    "SELECT EXISTS(SELECT 1 FROM room_invite
                WHERE sender=?1 AND recipient=?2 AND instance=?3 AND expires>unixepoch())",
                )?
                .query_row(params![player, recipient, active.id], |row| row.get(0))?;
            if !pending {
                messages.push(record_invite(db, player, recipient, &active)?.1);
            }
        }
    }
    Ok(messages)
}

fn update_instance(
    db: &Connection,
    player: i64,
    id: i64,
    action: &str,
    fields: &Fields,
) -> rusqlite::Result<Reply> {
    let Some(instance) = by_id(db, id)? else {
        return Ok(rooms::status(StatusCode::NOT_FOUND));
    };
    if crate::photon_gate::banned(db, player)? {
        return Ok(rooms::status(StatusCode::FORBIDDEN));
    }
    if action == "markprivate" {
        let room = rooms::by_id(db, instance.room)?.ok_or(rusqlite::Error::QueryReturnedNoRows)?;
        if !room.manages(db, player)? {
            return Ok(rooms::status(StatusCode::FORBIDDEN));
        }
        db.execute("UPDATE room_instance SET private=1 WHERE id=?1", [id])?;
    } else {
        let Some(policy) = field(fields, "policy")
            .and_then(|value| value.parse::<i64>().ok())
            .filter(|value| (0..=2).contains(value))
        else {
            return Ok(rooms::status(StatusCode::BAD_REQUEST));
        };
        if current(db, player)?.is_none_or(|current| current.id != id) {
            return Ok(rooms::status(StatusCode::FORBIDDEN));
        }
        db.execute(
            "UPDATE room_instance SET matchmaking_policy=?2 WHERE id=?1",
            params![id, policy],
        )?;
    }
    Ok(rooms::status(StatusCode::OK))
}

pub(crate) fn permissions(db: &Connection, player: i64) -> rusqlite::Result<Reply> {
    let instance = current(db, player)?;
    let mut role = 0;
    if let Some(instance) = &instance
        && let Some(room) = rooms::by_id(db, instance.room)?
    {
        role = if room.owner == Some(player) {
            255
        } else {
            rooms::role(db, room.id, player)?
        };
    }
    let instance_id = if in_orientation(db, player)? {
        Some(-2)
    } else {
        instance.map(|i| i.id)
    };
    let tiers = if role >= 30 {
        vec![0, 10, 20, 30, 255]
    } else if role == 0 {
        vec![0]
    } else {
        vec![0, role]
    };
    let mut permissions = Vec::new();
    for tier in tiers {
        for name in [
            "CAN_USE_MAKER_PEN",
            "CAN_USE_ROOM_RESET_BUTTON",
            "CAN_USE_DELETE_ALL_BUTTON",
            "CAN_SAVE_INVENTIONS",
            "CAN_SPAWN_INVENTIONS",
            "CAN_USE_PLAY_GIZMOS_TOGGLE",
        ] {
            permissions.push(json!({"Permission":name,"Role":tier,"Type":0,
                "Override":role<30,"Value":if tier>=30 {"True"} else {"False"}}));
        }
    }
    Ok(rooms::ok(
        json!({"Permissions":permissions,"PhotonAccessToken":"",
        "RoomInstanceId":instance_id}),
    ))
}

fn public_location(mut room: Value) -> Value {
    if room["roomInstanceId"] == -2 {
        return Value::Null;
    }
    if let Some(room) = room.as_object_mut() {
        room.remove("photonRegion");
        room.insert("photonRoomId".into(), json!(""));
        room.insert("dataBlob".into(), json!(""));
    }
    room
}

fn player_payload(
    db: &Connection,
    viewer: i64,
    target: i64,
    region: &str,
) -> rusqlite::Result<Value> {
    let visible = target == viewer || mutually_visible(db, viewer, target, true)?;
    let room = if visible {
        current_dto(db, target, region)?
    } else {
        Value::Null
    };
    let online: bool = visible && (!room.is_null() || db.query_row(
        "SELECT EXISTS(SELECT 1 FROM lobby_presence WHERE account_id=?1 AND expires>unixepoch())",
        [target], |r|r.get(0))?);
    let room = if target == viewer {
        room
    } else {
        public_location(room)
    };
    Ok(
        json!({"appVersion":"20230414","deviceClass":0,"errorCode":0,"isOnline":online,
        "playerId":target,"roomInstance":room,"statusVisibility":0,"vrMovementMode":1,"platform":0,
        "photonAuthToken":null,"photonRealtimeAppId":null,"photonVoiceAppId":null,"photonChatAppId":null,
        "photonRegion":null,"photonRoomId":null,"voiceConnectionInfo":null,"voiceServerId":null,"experiments":null}),
    )
}

pub(crate) async fn push_presence(store: &Store, hub: &notify::Hub, player: i64, region: String) {
    let result = store.with_db(move |db| {
        let mut stmt = db.prepare_cached("SELECT r.peer FROM relationship r JOIN relationship p
            ON p.owner=r.peer AND p.peer=r.owner
            WHERE r.owner=?1 AND r.kind=3 AND p.kind=3 AND r.ignored=0 AND p.ignored=0 LIMIT 100")?;
        let friends = stmt.query_map([player],|r|r.get::<_,i64>(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
        let room = public_location(current_dto(db,player,&region)?);
        Ok((friends,json!({"playerId":player,"appVersion":"20230414","deviceClass":0,
            "vrMovementMode":0,"statusVisibility":0,"isOnline":!room.is_null(),"roomInstance":room})))
    }).await;
    if let Ok((friends, message)) = result {
        for friend in friends {
            hub.send(friend, "PresenceUpdate", message.clone());
        }
    }
}

fn list_instances(db: &Connection, player: i64, id: i64) -> rusqlite::Result<Reply> {
    let Some(room) = rooms::by_id(db, id)? else {
        return Ok(rooms::status(StatusCode::NOT_FOUND));
    };
    if !room.manages(db, player)? {
        return Ok(rooms::status(StatusCode::FORBIDDEN));
    }
    let mut list = Vec::new();
    let mut instances =
        db.prepare_cached("SELECT * FROM room_instance WHERE room_id=?1 ORDER BY id")?;
    let mut players = db.prepare_cached("SELECT player FROM room_location WHERE instance=?1 AND expires>unixepoch() ORDER BY player")?;
    for instance in instances.query_map([id], Instance::from_row)? {
        let instance = instance?;
        let player_ids = players
            .query_map([instance.id], |r| r.get::<_, i64>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        if !player_ids.is_empty() {
            list.push(json!({"roomInstanceId":instance.id,"roomId":id,"subRoomId":room.subroom(),
                "isFull":player_ids.len() as i64>=room.capacity,"createdAt":instance.created,"playerIds":player_ids}));
        }
    }
    Ok(rooms::ok(json!(list)))
}

pub async fn serve(
    store: Option<&Store>,
    photon: std::sync::Arc<Photon>,
    trace: bool,
    hub: &notify::Hub,
    request: Request<Body>,
) -> Response {
    let (parts, query) = rooms::parts(request.uri());
    let path: Vec<_> = parts.iter().map(String::as_str).collect();
    if matches!(
        path.as_slice(),
        [
            "player",
            "login" | "logout" | "heartbeat" | "exclusivelogin" | "verifylock"
        ]
    ) {
        return presence::serve(store, hub, &photon.region, request).await;
    }
    let Some(store) = store else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };
    let Some(identity) = store.access_identity(request.headers()) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let player = identity.id;
    let method = request.method().clone();
    let movement = method == Method::POST && path.first() == Some(&"matchmake");
    let region = photon.region.clone();
    let mut targets = Vec::new();
    let request = if path.as_slice() == ["player"] {
        use axum::body::to_bytes;
        targets.extend(
            form_urlencoded::parse(request.uri().query().unwrap_or_default().as_bytes())
                .filter(|(key, _)| key.eq_ignore_ascii_case("id"))
                .filter_map(|(_, v)| v.parse::<i64>().ok())
                .take(100),
        );
        if method == Method::POST {
            let Ok(Ok(body)) = tokio::time::timeout(
                std::time::Duration::from_secs(10),
                to_bytes(request.into_body(), 8192),
            )
            .await
            else {
                return StatusCode::BAD_REQUEST.into_response();
            };
            targets.extend(
                form_urlencoded::parse(&body)
                    .filter(|(key, _)| key.eq_ignore_ascii_case("id"))
                    .filter_map(|(_, v)| v.parse::<i64>().ok())
                    .take(100),
            );
        }
        None
    } else {
        Some(request)
    };
    let mut party = Vec::new();
    let fields = if movement
        && path.get(1) == Some(&"room")
        && let Some(request) = request
    {
        match rooms::field_pairs(request).await {
            Ok(pairs) => {
                for (_, value) in pairs
                    .iter()
                    .filter(|(key, _)| key.eq_ignore_ascii_case("AdditionalPlayerIds"))
                {
                    if let Ok(id) = value.trim().parse::<i64>()
                        && id > 0
                        && id != player
                        && !party.contains(&id)
                    {
                        party.push(id);
                        if party.len() == MAX_PARTY_INVITEES {
                            break;
                        }
                    }
                }
                pairs.into_iter().collect()
            }
            Err(code) => return code.into_response(),
        }
    } else if matches!(method, Method::POST | Method::PUT) {
        match rooms::fields(request.unwrap_or_else(|| Request::new(Body::empty()))).await {
            Ok(fields) => fields,
            Err(code) => return code.into_response(),
        }
    } else {
        Fields::new()
    };
    if targets.is_empty() {
        targets.push(player);
    }
    let connection_info_request = path.as_slice() == ["player", "connection-info"];
    let secret = if connection_info_request {
        store.secret.clone()
    } else {
        Vec::new()
    };
    let result = store
        .with_db(move |db| {
            let path: Vec<_> = parts.iter().map(String::as_str).collect();
            let tx = db.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
            if !identity.is_current(&tx)? || !rooms::account_exists(&tx, player)? {
                return Ok((rooms::status(StatusCode::UNAUTHORIZED), Vec::new()));
            }
            let mut notifications = Vec::new();
            let reply = match (method, path.as_slice()) {
                (Method::POST, ["matchmake", ..]) => {
                    sweep(&tx)?;
                    let reply = matchmake(&tx, &path, player, &photon.region, &query, &fields)?;
                    if !party.is_empty() && reply.1["errorCode"] == 0 {
                        notifications = invite_party(&tx, player, &party)?;
                    }
                    reply
                }
                (Method::POST, ["invite"]) => {
                    sweep(&tx)?;
                    let (reply, message) = invite(&tx, player, &fields)?;
                    notifications.extend(message);
                    reply
                }
                (Method::GET | Method::POST, ["player"]) => {
                    let mut seen = std::collections::HashSet::new();
                    let mut players = Vec::new();
                    for target in targets.into_iter().take(100) {
                        if seen.insert(target) {
                            players.push(player_payload(&tx, player, target, &photon.region)?);
                        }
                    }
                    rooms::ok(json!(players))
                }
                (Method::GET, ["player", "connection-info"]) => {
                    connection_info(&tx, player, &photon, &secret)?
                }
                (Method::PUT, ["roominstance", id, "inprogress"]) => {
                    let id = numeric(id);
                    if current(&tx, player)?.is_none_or(|i| i.id != id) {
                        rooms::status(StatusCode::NOT_FOUND)
                    } else {
                        let value = field(&fields, "inProgress")
                            .is_some_and(|v| v.eq_ignore_ascii_case("true"));
                        tx.execute(
                            "UPDATE room_instance SET in_progress=?2 WHERE id=?1",
                            params![id, value],
                        )?;
                        rooms::status(StatusCode::OK)
                    }
                }
                (Method::POST, ["roominstance", id, action @ "markprivate"])
                | (Method::PUT, ["roominstance", id, action @ "matchpolicy"]) => {
                    update_instance(&tx, player, numeric(id), action, &fields)?
                }
                (Method::POST, ["roominstance", _, "reportjoinresult"]) => {
                    rooms::status(StatusCode::OK)
                }
                (Method::GET, ["room", id, "instances"]) => {
                    list_instances(&tx, player, numeric(id))?
                }
                _ => rooms::status(StatusCode::NOT_FOUND),
            };
            tx.commit()?;
            Ok((reply, notifications))
        })
        .await;
    match result {
        Ok((reply, notifications)) => {
            if trace && movement {
                let scene = reply.1["roomInstance"]["location"].as_str().unwrap_or("");
                // Emit only built-in identifiers we recognize, never user input.
                let scene = match scene {
                    rooms::DORM
                    | "c79709d8-a31b-48aa-9eb8-cc31ba9505e8"
                    | "a75f7547-79eb-47c6-8986-6767abcb4f92" => scene,
                    "" => "none",
                    _ => "other",
                };
                eprintln!(
                    "match stage=reservation result={} scene={scene}",
                    reply.1["errorCode"].as_i64().unwrap_or(-1)
                );
            }
            for message in notifications {
                hub.send(
                    message["ToPlayerId"].as_i64().unwrap_or_default(),
                    "2",
                    message,
                );
            }
            if movement && reply.1["errorCode"] == 0 {
                push_presence(store, hub, player, region).await;
            }
            let mut response = rooms::respond(Ok(reply));
            if connection_info_request {
                response.headers_mut().insert(
                    axum::http::header::CACHE_CONTROL,
                    axum::http::HeaderValue::from_static("no-store"),
                );
            }
            response
        }
        Err(error) => rooms::respond(Err(error)),
    }
}
