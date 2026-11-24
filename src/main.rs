#![recursion_limit = "256"]
mod accounts;
mod assets;
mod auth;
mod bootstrap;
mod catalogue;
mod live;
mod notify;
mod photon_gate;
mod presence;
mod rooms;
mod sessions;
mod settings;
mod social;

use axum::{
    body::Body,
    extract::State,
    http::{Method, Request, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::{env, io, path::PathBuf, sync::Arc, time::Duration};
use tokio::net::{TcpListener, TcpStream};

struct App {
    domain: String,
    discovery: axum::body::Bytes,
    config: axum::body::Bytes,
    aliases: std::collections::BTreeMap<String, String>,
    assets: PathBuf,
    settings: Option<Arc<settings::Store>>,
    signup: bool,
    trace_startup: bool,
    photon: Arc<sessions::Photon>,
    hub: Arc<notify::Hub>,
}

fn json_body(status: StatusCode, body: impl Into<Body>) -> Response {
    (
        status,
        [(header::CONTENT_TYPE, "application/json")],
        body.into(),
    )
        .into_response()
}

fn not_found() -> Response {
    json_body(
        StatusCode::NOT_FOUND,
        r#"{"success":false,"error":{"message":"not found"}}"#,
    )
}

async fn dispatch(State(app): State<Arc<App>>, request: Request<Body>) -> Response {
    let authority = request.headers().get(header::HOST).and_then(|value| {
        value
            .to_str()
            .ok()?
            .parse::<axum::http::uri::Authority>()
            .ok()
    });
    let host = authority.as_ref().map(|a| a.host().to_ascii_lowercase());
    let host = host.as_deref().unwrap_or_default();
    let service = if host == app.domain {
        ""
    } else if let Some(subdomain) = host
        .strip_suffix(&app.domain)
        .and_then(|s| s.strip_suffix('.'))
    {
        subdomain
    } else {
        return not_found();
    };
    let service = app.aliases.get(service).map_or(service, String::as_str);
    let method = request.method().clone();
    let head = method == Method::HEAD;
    let verb = if head { &Method::GET } else { &method };
    let path = request.uri().path();
    let trace = app
        .trace_startup
        .then(|| bootstrap::trace_label(service, path));
    let mut response = match (service, verb, path) {
        ("" | "ns", &Method::GET, "/") => json_body(StatusCode::OK, app.discovery.clone()),
        ("discovery", &Method::GET, "/sections/bulk") => catalogue::sections(request.uri()),
        ("commerce", &Method::GET, "/purchase/v1/hasspentmoney") => {
            json_body(StatusCode::OK, "false")
        }
        ("ai", &Method::GET, "/gameai/user/access") => {
            catalogue::account_feature(&app, request.headers(), None).await
        }
        ("api", &Method::GET, path) if catalogue::segment(path, "/iam/me/channels/").is_some() => {
            json_body(StatusCode::OK, "[]")
        }
        ("cards", &Method::GET, path)
            if catalogue::segment(path, "/iam/me/channels/").is_some() =>
        {
            catalogue::account_feature(
                &app,
                request.headers(),
                catalogue::segment(path, "/iam/me/channels/"),
            )
            .await
        }
        ("lists", &Method::GET, "/curatedlists") => catalogue::list(&app, request.uri()).await,
        ("studio", &Method::GET, "/cloud-builds") => catalogue::build(&app, request.uri()).await,
        ("clubs", &Method::GET, path) if catalogue::segment(path, "/club/").is_some() => {
            catalogue::club(&app, &path["/club/".len()..]).await
        }
        ("roomcomments", &Method::GET, path)
            if catalogue::segment(path, "/comments/get/").is_some() =>
        {
            catalogue::comments(&app, &path["/comments/get/".len()..], request.uri()).await
        }
        ("discovery", &Method::GET, "/") => {
            json_body(StatusCode::OK, r#"{"service":"discovery","status":"ok"}"#)
        }
        ("ai" | "chat" | "commerce" | "img" | "roomcomments" | "rooms", &Method::GET, "/") => {
            json_body(
                StatusCode::OK,
                format!(r#"{{"service":"{service}","status":"ok"}}"#),
            )
        }
        (
            "cards"
            | "leaderboard"
            | "link"
            | "lists"
            | "moderation"
            | "platformnotifications"
            | "storage"
            | "studio",
            &Method::GET,
            "/",
        ) => "hello, world!".into_response(),
        ("moderation", &Method::GET, "/voice/config") => {
            json_body(StatusCode::OK, r#"{"AccountId":"","ApiKey":""}"#)
        }
        ("playersettings", &Method::GET, "/") => json_body(
            StatusCode::OK,
            r#"{"service":"playersettings","status":"ok"}"#,
        ),
        ("playersettings", &Method::GET | &Method::PUT | &Method::DELETE, "/playersettings") => {
            match &app.settings {
                Some(store) => store.handle(request, head).await,
                None => StatusCode::UNAUTHORIZED.into_response(),
            }
        }
        ("accounts", &Method::GET, "/") => {
            json_body(StatusCode::OK, r#"{"service":"accounts","status":"ok"}"#)
        }
        ("accounts", _, path) if path.starts_with("/account/") => match &app.settings {
            Some(store) => accounts::serve(store, request, verb.clone(), app.signup).await,
            None => StatusCode::SERVICE_UNAVAILABLE.into_response(),
        },
        ("rooms", _, _) => {
            rooms::serve(app.settings.as_deref(), &app.hub, request, verb.clone()).await
        }
        ("match", _, _) => {
            sessions::serve(
                app.settings.as_deref(),
                app.photon.clone(),
                app.trace_startup,
                &app.hub,
                request,
            )
            .await
        }
        ("notify", _, _) => notify::serve(app.settings.as_deref(), app.hub.clone(), request).await,
        ("auth", &Method::GET | &Method::POST, path)
            if path
                .strip_prefix("/cachedlogin/forplatformid/")
                .is_some_and(|suffix| {
                    suffix.split_once('/').is_some_and(|(platform, id)| {
                        !platform.is_empty() && !id.is_empty() && !id.contains('/')
                    })
                }) =>
        {
            // This deployment has no platform links. The picker must see an empty
            // array for an unlinked identity so the client can choose manual login.
            json_body(StatusCode::OK, "[]")
        }
        ("auth", &Method::POST, "/account/me/changepassword") => match &app.settings {
            Some(store) => auth::change_password(store, request).await,
            None => StatusCode::UNAUTHORIZED.into_response(),
        },
        ("auth", &Method::POST, "/connect/token") => match &app.settings {
            Some(store) => auth::token(store, request, app.signup).await,
            None => StatusCode::SERVICE_UNAVAILABLE.into_response(),
        },
        ("api", _, path)
            if path.starts_with("/api/relationships/")
                || matches!(
                    path,
                    "/api/messages/v2/get"
                        | "/api/messages/v3/delete"
                        | "/api/messages/v1/friendOnlineStatus"
                ) =>
        {
            match &app.settings {
                Some(store) => social::serve(store, request, verb).await,
                None => StatusCode::UNAUTHORIZED.into_response(),
            }
        }
        ("api" | "accounts", _, _) => bootstrap::serve(&app, service, verb, request.uri()),
        ("cdn", &Method::GET, _) => assets::serve(&app.assets, request, head).await,
        _ => not_found(),
    };
    if let Some((service, route)) = trace {
        let method = match method {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::HEAD => "HEAD",
            _ => "OTHER",
        };
        eprintln!(
            "startup {service} {method} {route} {}",
            response.status().as_u16()
        );
    }
    if head {
        *response.body_mut() = Body::empty();
    }
    response
}

struct BoundedListener {
    tcp: TcpListener,
    permits: Arc<tokio::sync::Semaphore>,
}

struct Connection {
    tcp: TcpStream,
    _permit: tokio::sync::OwnedSemaphorePermit,
}

impl tokio::io::AsyncRead for Connection {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        context: &mut std::task::Context<'_>,
        buffer: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        std::pin::Pin::new(&mut self.tcp).poll_read(context, buffer)
    }
}

impl tokio::io::AsyncWrite for Connection {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        bytes: &[u8],
    ) -> std::task::Poll<io::Result<usize>> {
        std::pin::Pin::new(&mut self.tcp).poll_write(cx, bytes)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        std::pin::Pin::new(&mut self.tcp).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        std::pin::Pin::new(&mut self.tcp).poll_shutdown(cx)
    }
}

impl axum::serve::Listener for BoundedListener {
    type Io = Connection;
    type Addr = std::net::SocketAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        let permit = self
            .permits
            .clone()
            .acquire_owned()
            .await
            .expect("semaphore remains open");
        let (tcp, addr) = axum::serve::Listener::accept(&mut self.tcp).await;
        (
            Connection {
                tcp,
                _permit: permit,
            },
            addr,
        )
    }

    fn local_addr(&self) -> io::Result<Self::Addr> {
        self.tcp.local_addr()
    }
}

async fn stop() {
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("install SIGTERM handler");
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {},
        _ = terminate.recv() => {},
    }
}

fn valid_domain(domain: &str) -> bool {
    domain.len() <= 220
        && domain.split('.').all(|part| {
            !part.is_empty()
                && part.len() <= 63
                && !part.starts_with('-')
                && !part.ends_with('-')
                && part.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .max_blocking_threads(4)
        .thread_keep_alive(Duration::from_secs(10))
        .build()?
        .block_on(run())
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let domain = env::var("KROMAREC_DOMAIN")
        .unwrap_or_else(|_| "rec.test".into())
        .to_ascii_lowercase();
    if !valid_domain(&domain) {
        return Err("KROMAREC_DOMAIN must be a DNS name without a scheme or port".into());
    }
    let overrides = env::var("KROMAREC_SUBDOMAINS").unwrap_or_else(|_| "{}".into());
    let overrides: std::collections::BTreeMap<String, String> = serde_json::from_str(&overrides)?;
    if overrides.values().any(|value| !valid_domain(value)) {
        return Err(
            "KROMAREC_SUBDOMAINS values must be DNS names relative to KROMAREC_DOMAIN".into(),
        );
    }
    let services: std::collections::BTreeMap<String, String> =
        serde_json::from_str(include_str!("../compat/service-hosts.json"))?;
    let mut aliases = std::collections::BTreeMap::new();
    let mut discovery = serde_json::Map::new();
    for (label, sub) in services.into_iter().chain([("".into(), "ns".into())]) {
        if matches!(
            label.as_str(),
            "DataCollection" | "BugReporting" | "GameLogs"
        ) {
            continue;
        }
        let host = overrides.get(&sub).unwrap_or(&sub).to_ascii_lowercase();
        if !label.is_empty() {
            discovery.insert(label, json!(format!("https://{host}.{domain}")));
        }
        if aliases.insert(host, sub).is_some() {
            return Err("multiple services resolve to the same subdomain".into());
        }
    }
    let assets = env::var("KROMAREC_ASSETS").unwrap_or_else(|_| "assets".into());
    let settings = match env::var("KROMAREC_JWT_SECRET") {
        Ok(secret) if secret.len() >= 32 => {
            let path = env::var("KROMAREC_DB").unwrap_or_else(|_| "kromarec.sqlite3".into());
            Some(Arc::new(settings::Store::open(&path, secret.into_bytes())?))
        }
        Ok(_) => return Err("KROMAREC_JWT_SECRET must contain at least 32 bytes".into()),
        Err(env::VarError::NotPresent) => None,
        Err(error) => return Err(error.into()),
    };
    let bind = env::var("KROMAREC_BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let cap = env::var("KROMAREC_MAX_CONNECTIONS")
        .unwrap_or_else(|_| "128".into())
        .parse::<usize>()?;
    if !(1..=4096).contains(&cap) {
        return Err("KROMAREC_MAX_CONNECTIONS must be between 1 and 4096".into());
    }
    let mut config: serde_json::Value =
        serde_json::from_str(include_str!("../compat/api-config-v2.json"))?;
    let www = overrides.get("www").map_or("www", String::as_str);
    config["ShareBaseUrl"] = json!(format!("https://{www}.{domain}/{{0}}"));
    let app = Arc::new(App {
        photon: Arc::new(sessions::Photon::from_env()),
        hub: Arc::new(notify::Hub::default()),
        domain,
        config: serde_json::to_vec(&config)?.into(),
        aliases,
        discovery: serde_json::to_vec(&discovery)?.into(),
        assets: tokio::fs::canonicalize(assets).await?,
        settings,
        trace_startup: env::var("KROMAREC_TRACE_STARTUP").as_deref() == Ok("true"),
        signup: env::var("KROMAREC_PASSWORD_SIGNUP").as_deref() == Ok("true"),
    });
    let _photon_gate = photon_gate::start(&app)?;
    let routes = axum::Router::new().fallback(dispatch).with_state(app);
    let tcp = TcpListener::bind(&bind).await?;
    eprintln!("KromaRec listening on {}", tcp.local_addr()?);
    let (trigger, shutdown) = tokio::sync::oneshot::channel();
    let server = axum::serve(
        BoundedListener {
            tcp,
            permits: Arc::new(tokio::sync::Semaphore::new(cap)),
        },
        routes,
    )
    .with_graceful_shutdown(async {
        let _ = shutdown.await;
    });
    let mut server = std::pin::pin!(std::future::IntoFuture::into_future(server));
    tokio::select! {
        result = &mut server => result?,
        _ = stop() => {
            let _ = trigger.send(());
            if let Ok(result) = tokio::time::timeout(Duration::from_secs(10), server).await {
                result?;
            }
        }
    }
    Ok(())
}
