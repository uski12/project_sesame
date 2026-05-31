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
    sync::{Arc, RwLock},
    time::{Duration, Instant},
    env,
};

const AUTH_DURATION_SECONDS: u64 = 60;

#[derive(Clone)]
struct AppState {
    authorized_ips: Arc<RwLock<HashMap<String, Instant>>>,
    config: EnvConfig,
}

#[derive(Deserialize)]
struct KnockRequest {
    passphrase: String,
}

#[derive(Serialize)]
struct KnockResponse {
    status: String,
}

#[derive(Clone)]
struct EnvConfig {
    gateway_host: String,
    gateway_port: u16,

    server_host: String,
    server_port: u16,

    passphrase: String,
}

impl EnvConfig {
    fn load_env() -> Self {
        from_path("../../.env").ok();
        dotenv().ok();

        Self {
            gateway_host: env::var("GATEWAY_HOST")
                .expect("GATEWAY_HOST field missing"),

            gateway_port: env::var("GATEWAY_PORT")
                .expect("GATEWAY_PORT field missing")
                .parse()
                .expect("Invalid GATEWAY_PORT field"),

            server_host: env::var("SERVER_HOST")
                .expect("SERVER_HOST field missing"),

            server_port: env::var("SERVER_PORT")
                .expect("SERVER_PORT field missing")
                .parse()
                .expect("Invalid SERVER_PORT field"),

            passphrase: env::var("PASSPHRASE")
                .expect("PASSPHRASE field missing"),
        }
    }
}


#[tokio::main]
async fn main() {
    let config = EnvConfig::load_env();

    let state = AppState {
        authorized_ips: Arc::new(RwLock::new(HashMap::new())),
        config: config.clone(),
    };

    let app = Router::new()
    .route("/knock", post(knock_handler))
    .route("/dashboard", get(proxy_dashboard))
    .layer(middleware::from_fn(req_logger))
    .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.gateway_host, config.gateway_port))
    .await
    .unwrap();

    println!("Gateway listening on :{}", config.gateway_port);

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

    if payload.passphrase != state.config.passphrase {
        fake_failure().await;

        return (
            StatusCode::NOT_FOUND,
            Json(KnockResponse {
                status: "not found".into(),
            }),
        );
    }

    let client_ip = addr.ip().to_string();

    let expiry =
    Instant::now() + Duration::from_secs(AUTH_DURATION_SECONDS);

    state
    .authorized_ips
    .write()
    .unwrap()
    .insert(client_ip.clone(), expiry);

    println!("Authorized {}", client_ip);

    (
        StatusCode::OK,
     Json(KnockResponse {
         status: "authorized".into(),
     }),
    )
}

async fn proxy_dashboard(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Response {

    let client_ip = addr.ip().to_string();

    let authorized = {
        let map = state.authorized_ips.read().unwrap();

        if let Some(expiry) = map.get(&client_ip) {
            *expiry > Instant::now()
        } else {
            false
        }
    };

    if !authorized {
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
            StatusCode::BAD_GATEWAY.into_response()
        }
    }
}

async fn fake_failure() {
    let delay =
    rand::thread_rng().gen_range(500..3000);

    tokio::time::sleep(
        Duration::from_millis(delay)
    ).await;
}


async fn req_logger(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {

    println!("==========================");
    println!("IP        : {}", addr.ip());
    println!("PORT      : {}", addr.port());
    println!("METHOD    : {}", request.method());
    println!("URI       : {}", request.uri());
    println!("VERSION   : {:?}", request.version());

    println!("HEADERS:");
    for (key, value) in request.headers() {
        println!("  {}: {:?}", key, value);
    }

    println!("==========================");

    next.run(request).await
}
