use axum::{
    extract::{ConnectInfo, State, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    middleware::Next,
    middleware,
    Json, Router,
};

use rand::Rng;
use serde::{Deserialize, Serialize};

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

const PASSPHRASE: &str = "super_secret_passphrase";
const AUTH_DURATION_SECONDS: u64 = 60;

#[derive(Clone)]
struct AppState {
    authorized_ips: Arc<Mutex<HashMap<String, Instant>>>,
}

#[derive(Deserialize)]
struct KnockRequest {
    passphrase: String,
}

#[derive(Serialize)]
struct KnockResponse {
    status: String,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        authorized_ips: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
    .route("/knock", post(knock_handler))
    .route("/dashboard", get(proxy_dashboard))
    .layer(middleware::from_fn(req_logger))
    .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8009")
    .await
    .unwrap();

    println!("Gateway listening on :8009");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn knock_handler(State(state): State<AppState>,
                       ConnectInfo(addr): ConnectInfo<SocketAddr>,
                       Json(payload): Json<KnockRequest>
                       ) -> impl IntoResponse {

    if payload.passphrase != PASSPHRASE {
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
    .lock()
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
        let map = state.authorized_ips.lock().unwrap();

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
    reqwest::get("http://127.0.0.1:3000/")
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
