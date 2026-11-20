//! Local authorization channel used by the pinned Luxon extension.
use crate::{App, sessions, settings::Store};
use serde::Deserialize;
use std::{io, net::SocketAddrV4, sync::Arc, time::Duration};
use subtle::ConstantTimeEq;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{OwnedSemaphorePermit, Semaphore},
    task::JoinHandle,
};

#[derive(Deserialize)]
struct Ticket {
    sub: String,
    aud: String,
    exp: i64,
    #[serde(rename = "rn.env")]
    environment: String,
    kromarec_use: String,
    kromarec_access_generation: i64,
    kromarec_photon_generation: i64,
}

pub struct Gate {
    task: JoinHandle<()>,
}

impl Drop for Gate {
    fn drop(&mut self) {
        self.task.abort();
    }
}

pub fn start(app: &Arc<App>) -> io::Result<Option<Gate>> {
    let Some(bind) = std::env::var_os("KROMAREC_PHOTON_GATE_ADDR") else {
        return Ok(None);
    };
    let invalid = |msg| io::Error::new(io::ErrorKind::InvalidInput, msg);
    let Some(store) = app.settings.clone() else {
        return Err(invalid("Photon gate requires RecNet authentication"));
    };
    if app.photon.realtime.is_empty() {
        return Err(invalid("Photon gate requires a Realtime application ID"));
    }
    let bind: SocketAddrV4 = bind
        .to_str()
        .and_then(|v| v.parse().ok())
        .filter(|v: &SocketAddrV4| v.ip() == &std::net::Ipv4Addr::LOCALHOST && v.port() != 0)
        .ok_or_else(|| invalid("Photon gate must bind to 127.0.0.1"))?;
    let key = std::env::var("KROMAREC_PHOTON_GATE_KEY")
        .ok()
        .filter(|v| v.len() == 64 && v.bytes().all(|b| b.is_ascii_hexdigit()))
        .ok_or_else(|| invalid("Photon gate requires a randomly generated 64-hex-digit key"))?;
    let listener = std::net::TcpListener::bind(bind)?;
    listener.set_nonblocking(true)?;
    let listener = TcpListener::from_std(listener)?;
    let key: Arc<str> = key.into();
    let photon = app.photon.clone();
    let trace = app.trace_startup;
    let task = tokio::spawn(async move {
        let slots = Arc::new(tokio::sync::Semaphore::new(16));
        let actors = Arc::new(Semaphore::new(128));
        while let Ok((stream, _)) = listener.accept().await {
            if let Ok(slot) = slots.clone().try_acquire_owned() {
                let actors = actors.clone();
                let store = store.clone();
                let photon = photon.clone();
                let key = key.clone();
                tokio::spawn(async move {
                    let mut stream = stream;
                    let admitted = tokio::time::timeout(
                        Duration::from_millis(750),
                        request(&mut stream, &store, &photon, &key, &actors, trace),
                    )
                    .await;
                    drop(slot);
                    if let Ok(Ok(Some(actor))) = admitted {
                        actor.monitor(stream, &store).await;
                    }
                });
            }
        }
    });
    Ok(Some(Gate { task }))
}

async fn request(
    stream: &mut TcpStream,
    store: &Store,
    photon: &sessions::Photon,
    key: &str,
    actors: &Arc<Semaphore>,
    trace: bool,
) -> io::Result<Option<Actor>> {
    // Legacy messages start with a hex key and end at EOF. A live Game
    // admission starts with NUL and a big-endian u16 payload length so the
    // authenticated TCP connection can stay open for its actor's lifetime.
    let first = stream.read_u8().await?;
    let watch = first == 0;
    let mut bytes = Vec::new();
    if watch {
        let size = stream.read_u16().await? as usize;
        if size > 12_288 {
            return Ok(None);
        }
        bytes.resize(size, 0);
        stream.read_exact(&mut bytes).await?;
    } else {
        bytes.push(first);
        (&mut *stream).take(12_288).read_to_end(&mut bytes).await?;
    }
    let mut decision = None;
    let mut stage = "invalid";
    if bytes.len() <= 12_288
        && let Some((offered, payload)) = bytes.split_at_checked(64)
        && bool::from(offered.ct_eq(key.as_bytes()))
        && let Some(fields) = payload
            .strip_prefix(&[0])
            .and_then(|v| std::str::from_utf8(v).ok())
    {
        let mut parts = fields.split('\0');
        if let (Some(kind), Some(app), Some(token), Some(room), None) = (
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ) {
            stage = match (watch, kind) {
                (true, "watch") => "game-admission",
                (false, "auth") => "authenticate",
                (false, "name-auth") => "name-auth",
                (false, "master-auth") => "master-auth",
                (false, "game-auth") => "game-auth",
                (false, "join") => "room-admission",
                _ => "invalid",
            };
            if stage != "invalid" {
                decision = authorize(store, photon, kind, app, token, room, actors).await;
            }
        }
    }
    if trace {
        eprintln!("photon stage={stage} admitted={}", decision.is_some());
    }
    let Some((id, actor)) = decision else {
        stream.write_all(b"0").await?;
        return Ok(None);
    };
    stream.write_all(id.as_bytes()).await?;
    if watch {
        stream.write_all(b"\n").await?;
    }
    Ok(actor)
}

struct Actor {
    _slot: OwnedSemaphorePermit,
    player: i64,
    room: Arc<str>,
    lease: crate::live::Lease,
}

impl Actor {
    async fn monitor(mut self, mut stream: TcpStream, store: &Store) {
        // Direct operator SQL bans, room deletion and reservation expiry have
        // no application callback. Check only while this actor is connected.
        loop {
            tokio::select! {
                biased;
                _ = self.lease.revoked() => return,
                _ = stream.read_u8() => return,
                _ = tokio::time::sleep(Duration::from_secs(5)) => {}
            }
            let player = self.player;
            let room = self.room.clone();
            let check = store.with_db(move |db| {
                Ok(!banned(db, player)?
                    && sessions::photon_room(db, player)?.as_deref() == Some(&*room))
            });
            tokio::select! {
                _ = self.lease.revoked() => return,
                result = tokio::time::timeout(Duration::from_secs(2), check) => {
                    if !matches!(result, Ok(Ok(true))) { return; }
                    // A stalled monitor must not silently extend a Game lease.
                    if !matches!(stream.try_write(b"+"), Ok(1)) { return; }
                }
            }
        }
    }
}

async fn authorize(
    store: &Store,
    photon: &sessions::Photon,
    kind: &str,
    app: &str,
    token: &str,
    room: &str,
    actors: &Arc<Semaphore>,
) -> Option<(String, Option<Actor>)> {
    if app != photon.realtime
        || !matches!(
            (kind, room.is_empty()),
            ("auth" | "name-auth" | "master-auth" | "game-auth", true) | ("join" | "watch", false)
        )
    {
        return None;
    }
    let ticket: Ticket = store.verify_signed(token)?;
    let player: i64 = ticket.sub.parse().ok()?;
    if player <= 0
        || ticket.sub != player.to_string()
        || ticket.aud != app
        || ticket.environment != "prod"
        || ticket.kromarec_use != "photon-realtime"
    {
        return None;
    }
    let slot = if kind == "watch" {
        Some(actors.clone().try_acquire_owned().ok()?)
    } else {
        None
    };
    let registry = store.live.clone();
    let room = room.to_owned();
    store
        .with_db(move |db| {
            let live: bool = db
                .prepare_cached(
                    "SELECT EXISTS(SELECT 1 FROM local_account WHERE id=?1 AND access_generation=?2
                AND photon_generation=?3 AND ?4>unixepoch())",
                )?
                .query_row(
                    rusqlite::params![
                        player,
                        ticket.kromarec_access_generation,
                        ticket.kromarec_photon_generation,
                        ticket.exp
                    ],
                    |row| row.get(0),
                )?;
            if !live || banned(db, player)? {
                return Ok(None);
            }
            if !room.is_empty()
                && sessions::photon_room(db, player)?.as_deref() != Some(room.as_str())
            {
                return Ok(None);
            }
            let actor = slot.map(|slot| Actor {
                _slot: slot,
                player,
                room: room.into(),
                lease: registry.admit(player),
            });
            Ok(Some((ticket.sub, actor)))
        })
        .await
        .ok()
        .flatten()
}

pub(crate) fn banned(db: &rusqlite::Connection, player: i64) -> rusqlite::Result<bool> {
    db.prepare_cached(
        "SELECT EXISTS(SELECT 1 FROM photon_ban
        WHERE player=?1 AND (expires IS NULL OR expires>unixepoch()))",
    )?
    .query_row([player], |r| r.get(0))
}
