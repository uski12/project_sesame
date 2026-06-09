use axum::{
    extract::{ConnectInfo, State, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    middleware::Next,
    middleware,
    Json, Router,
};

use dotenvy::{
    from_path,
    dotenv,
};
use rand::Rng;
use serde::{Deserialize, Serialize};

use std::{
    collections::HashMap,
    net::SocketAddr,
    net::IpAddr,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
    env,
};

use tracing::{
    info,
    warn,
    error,
    debug,
};

#[derive(Debug, Default)]
struct FailedIpInfo {
    attempts: u8,
    blocked_expiry: Option<Instant>,
}


#[derive(Clone)]
struct AppState {
    authorized_ips: Arc<RwLock<HashMap<IpAddr, Instant>>>,
    failed_ips: Arc<RwLock<HashMap<IpAddr, FailedIpInfo>>>,
    used_nonces: Arc<RwLock<HashMap<String, Instant>>>,
    config: EnvConfig,
}

#[derive(Deserialize)]
struct KnockRequest {
    passphrase: String,
    // timestamp: u64,
    // nonce: String,
}

#[derive(Serialize)]
struct KnockResponse {
    status: String,
}

#[derive(Clone)]
struct EnvConfig {
    gateway_host: IpAddr,
    gateway_port: u16,

    server_host: IpAddr,
    server_port: u16,

    passphrase: String,
    auth_dur: u16,

    max_failed_attempts: u8,
    timeout_dur: u16,
}

impl EnvConfig {
    fn load_env() -> Self {
        from_path("../../.env").ok();
        dotenv().ok();

        Self {
            gateway_host: env::var("GATEWAY_HOST")
                .expect("GATEWAY_HOST field missing")
                .parse()
                .expect("Invalid GATEWAY_HOST field"),

            gateway_port: env::var("GATEWAY_PORT")
                .expect("GATEWAY_PORT field missing")
                .parse()
                .expect("Invalid GATEWAY_PORT field"),


            server_host: env::var("SERVER_HOST")
                .expect("SERVER_HOST field missing")
                .parse()
                .expect("Invalid SERVER_HOST field"),

            server_port: env::var("SERVER_PORT")
                .expect("SERVER_PORT field missing")
                .parse()
                .expect("Invalid SERVER_PORT field"),


            passphrase: env::var("PASSPHRASE")
                .expect("PASSPHRASE field missing"),

            auth_dur: env::var("AUTH_DURATION_SECONDS")
                .expect("AUTH_DURATION_SECONDS field missing")
                .parse()
                .expect("Invalid AUTH_DURATION_SECONDS field"),


            max_failed_attempts: env::var("MAX_FAILED_ATTEMPTS")
                .expect("MAX_FAILED_ATTEMPTS field missing")
                .parse()
                .expect("Invalid MAX_FAILED_ATTEMPTS field"),

            timeout_dur: env::var("TIMEOUT_DURATION_SECONDS")
                .expect("TIMEOUT_DURATION_SECONDS field missing")
                .parse()
                .expect("Invalid TIMEOUT_DURATION_SECONDS field"),
        }
    }
}


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = EnvConfig::load_env();

    info!("Starting server...");

    let state = AppState {
        authorized_ips: Arc::new(RwLock::new(HashMap::new())),
        failed_ips: Arc::new(RwLock::new(HashMap::new())),
        used_nonces: Arc::new(RwLock::new(HashMap::new())),
        config: config.clone(),
    };

    let app = Router::new()
    .route("/knock", post(knock_handler))
    .route("/dashboard", get(proxy_dashboard))
    .layer(middleware::from_fn_with_state(state.clone(), req_logger))
    .with_state(state);


    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.gateway_host, config.gateway_port))
    .await
    .unwrap();

    info!("Gateway listening on PORT: {}", config.gateway_port);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn knock_handler(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<KnockRequest>
) -> impl IntoResponse {

    let client_ip = addr.ip();

    {
        let mut map = state.failed_ips.write().unwrap();

        let info = map
        .entry(client_ip)
        .or_default();

        if let Some(expiry) = info.blocked_expiry {
            if Instant::now() < expiry {
                warn!("Blocked request! IP: {}", client_ip);
                return (
                    StatusCode::NOT_FOUND,
                    Json(KnockResponse {
                        status: "not found".into(),
                    }),
                );
            }
        }
    }


    if payload.passphrase != state.config.passphrase {
        warn!("Failed knock! IP: {}, attempt: {}", client_ip, state.failed_ips.read().unwrap().get(&addr.ip()).map(|v| v.attempts).unwrap_or(0),);
        {
            let mut map = state.failed_ips.write().unwrap();

            let info = map
            .entry(client_ip)
            .or_default();

            info.attempts += 1;

            if info.attempts >= state.config.max_failed_attempts {
                info.blocked_expiry = Some(Instant::now() + Duration::from_secs(state.config.timeout_dur as u64 * (info.attempts - state.config.max_failed_attempts + 1) as u64));
            }
        }

        fake_failure().await;

        return (
            StatusCode::NOT_FOUND,
            Json(KnockResponse {
                status: "not found".into(),
            }),
        );
    }

    state
    .failed_ips.write()
    .unwrap()
    .remove(&client_ip);

    let auth_expiry = Instant::now() + Duration::from_secs(state.config.auth_dur.into());

    state
    .authorized_ips
    .write()
    .unwrap()
    .insert(client_ip, auth_expiry);

    info!("Authorized {}", client_ip);

    (
        StatusCode::OK,
        Json(KnockResponse {
            status: "Authorized".into(),
        }),
    )
}

async fn proxy_dashboard(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Response {

    let client_ip = addr.ip();

    let authorized = {
        let map = state.authorized_ips.read().unwrap();

        if let Some(expiry) = map.get(&client_ip) {
            *expiry > Instant::now()
        } else {
            false
        }
    };

    if !authorized {
        warn!("Unauthorised service request! IP: {}", client_ip);
        fake_failure().await;

        return StatusCode::BAD_GATEWAY.into_response();
    }

    let response =
    reqwest::get(format!("http://{}:{}", state.config.server_host, state.config.server_port))
    .await;

    match response {
        Ok(resp) => {
            let body =
            resp.text().await.unwrap_or_default();

            body.into_response()
        }

        Err(_) => {
            error!("Internal server not found!");
            StatusCode::BAD_GATEWAY.into_response()
        }
    }
}

async fn fake_failure() {
    let delay =
    rand::thread_rng().gen_range(1000..5000);

    tokio::time::sleep(
        Duration::from_millis(delay)
    ).await;
}


async fn req_logger(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {

    info!(
        ip = %addr.ip(),
        method = %request.method(),
        uri = %request.uri(),
        "Request received: "
    );

    debug!(headers= ?request.headers());

    next.run(request).await
}
