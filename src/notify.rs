use crate::{json_body, not_found, sessions, settings::Store};
use axum::{
    body::Body,
    extract::{
        FromRequestParts,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::{Method, Request, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::{Semaphore, broadcast};

const MAX_SOCKETS: usize = 128;
const RS: char = '\u{001e}';

pub struct Hub {
    players: Mutex<HashMap<i64, broadcast::Sender<String>>>,
    slots: Arc<Semaphore>,
}

impl Default for Hub {
    fn default() -> Self {
        Self {
            players: Mutex::new(HashMap::new()),
            slots: Arc::new(Semaphore::new(MAX_SOCKETS)),
        }
    }
}

impl Hub {
    pub fn send(&self, player: i64, kind: &str, data: Value) {
        let players = self.players.lock().expect("hub lock");
        if let Some(channel) = players.get(&player) {
            let payload = json!({"Id":kind,"Msg":data}).to_string();
            let _ = channel.send(invocation("Notification", json!([payload])));
        }
    }

    fn subscribe(&self, player: i64) -> Subscription<'_> {
        let mut players = self.players.lock().expect("hub lock");
        let receiver = players
            .entry(player)
            .or_insert_with(|| broadcast::channel(16).0)
            .subscribe();
        Subscription {
            hub: self,
            player,
            receiver: Some(receiver),
        }
    }
}

struct Subscription<'a> {
    hub: &'a Hub,
    player: i64,
    receiver: Option<broadcast::Receiver<String>>,
}

impl Drop for Subscription<'_> {
    fn drop(&mut self) {
        let mut players = self.hub.players.lock().expect("hub lock");
        self.receiver.take();
        if players
            .get(&self.player)
            .is_some_and(|sender| sender.receiver_count() == 0)
        {
            players.remove(&self.player);
        }
    }
}

fn frame(value: Value) -> String {
    format!("{value}{RS}")
}

fn invocation(target: &str, arguments: Value) -> String {
    frame(json!({"type":1,"target":target,"arguments":arguments}))
}

pub async fn serve(store: Option<&Store>, hub: Arc<Hub>, request: Request<Body>) -> Response {
    let (mut parts, _) = request.into_parts();
    let query: crate::rooms::Fields =
        form_urlencoded::parse(parts.uri.query().unwrap_or_default().as_bytes())
            .into_owned()
            .collect();
    match (&parts.method, parts.uri.path()) {
        (&Method::POST, "/hub/v1/negotiate") => {
            let Ok(id) = sessions::uuid() else {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            };
            let version = query
                .get("negotiateVersion")
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(0);
            json_body(
                StatusCode::OK,
                json!({"negotiateVersion":version,"connectionId":id,"connectionToken":id,
                "availableTransports":[{"transport":"WebSockets","transferFormats":["Text"]}]})
                .to_string(),
            )
        }
        (&Method::GET, "/hub/v1") => {
            if !parts.headers.contains_key(header::AUTHORIZATION)
                && let Some(token) = query.get("access_token")
                && let Ok(value) = format!("Bearer {token}").parse()
            {
                parts.headers.insert(header::AUTHORIZATION, value);
            }
            let Some(store) = store else {
                return StatusCode::UNAUTHORIZED.into_response();
            };
            let Some(identity) = store.access_identity(&parts.headers) else {
                return StatusCode::UNAUTHORIZED.into_response();
            };
            let player = identity.id;
            let expires = identity.expires;
            let live = store.live.clone();
            let Ok(Some(mut lease)) = store
                .with_db(move |db| Ok(identity.is_current(db)?.then(|| live.admit(player))))
                .await
            else {
                return StatusCode::UNAUTHORIZED.into_response();
            };
            let Ok(upgrade) = WebSocketUpgrade::from_request_parts(&mut parts, &()).await else {
                return json_body(
                    StatusCode::UPGRADE_REQUIRED,
                    r#"{"error":"Expected a WebSocket upgrade request"}"#,
                );
            };
            let Ok(slot) = hub.slots.clone().try_acquire_owned() else {
                return StatusCode::SERVICE_UNAVAILABLE.into_response();
            };
            upgrade
                .max_message_size(8192)
                .max_frame_size(8192)
                .read_buffer_size(4096)
                .write_buffer_size(0)
                .max_write_buffer_size(32768)
                .on_upgrade(move |socket| async move {
                    let _slot = slot;
                    let expiry = async {
                        if let Some(seconds) = expires {
                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default();
                            // Long-lived legacy tokens must not overflow Tokio's deadline.
                            let remaining = Duration::from_secs(seconds).saturating_sub(now);
                            tokio::time::sleep(remaining).await;
                        } else {
                            std::future::pending::<()>().await;
                        }
                    };
                    tokio::select! {
                        biased;
                        _ = lease.revoked() => {}
                        _ = expiry => {}
                        _ = run(socket, &hub, player) => {}
                    }
                })
        }
        _ => not_found(),
    }
}

async fn write(socket: &mut WebSocket, text: String) -> bool {
    matches!(
        tokio::time::timeout(
            Duration::from_secs(5),
            socket.send(Message::Text(text.into()))
        )
        .await,
        Ok(Ok(()))
    )
}

async fn run(mut socket: WebSocket, hub: &Hub, player: i64) {
    let Ok(Some(Ok(Message::Text(text)))) =
        tokio::time::timeout(Duration::from_secs(10), socket.recv()).await
    else {
        return;
    };
    let mut records = text.split_terminator(RS);
    let handshake = text
        .ends_with(RS)
        .then(|| records.next())
        .flatten()
        .and_then(|record| serde_json::from_str::<Value>(record).ok());
    if !handshake.is_some_and(|value| value["protocol"] == "json" && value["version"] == 1) {
        write(
            &mut socket,
            frame(json!({"error":"Expected SignalR JSON protocol version 1"})),
        )
        .await;
        return;
    }
    let mut subscription = hub.subscribe(player);
    if !write(&mut socket, format!("{{}}{RS}")).await
        || !write(&mut socket, invocation("OnConnect", json!([]))).await
    {
        return;
    }
    if !handle_records(&mut socket, records).await {
        return;
    }
    let receiver = subscription.receiver.as_mut().expect("active subscription");
    loop {
        tokio::select! {
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        if !text.ends_with(RS)
                            || !handle_records(&mut socket, text.split_terminator(RS)).await {
                            return;
                        }
                    }
                    Some(Ok(Message::Ping(_) | Message::Pong(_))) => {}
                    _ => return,
                }
            }
            notification = receiver.recv() => {
                match notification {
                    Ok(text) => if !write(&mut socket,text).await { return; },
                    Err(_) => return,
                }
            }
            _ = tokio::time::sleep(Duration::from_secs(15)) => {
                if !write(&mut socket,frame(json!({"type":6}))).await { return; }
            }
        }
    }
}

// One WebSocket message can carry several independent SignalR records.
async fn handle_records(
    socket: &mut WebSocket,
    records: std::str::SplitTerminator<'_, char>,
) -> bool {
    for record in records.filter(|record| !record.is_empty()) {
        let Ok(message) = serde_json::from_str::<Value>(record) else {
            return false;
        };
        let reply = match message["type"].as_i64() {
            Some(6) => json!({"type":6}),
            Some(7) => return false,
            Some(1) => match message["invocationId"].as_str() {
                Some(id) => {
                    json!({"type":3,"invocationId":id,"error":"Unsupported hub invocation"})
                }
                None => continue,
            },
            _ => continue,
        };
        if !write(socket, frame(reply)).await {
            return false;
        }
    }
    true
}
