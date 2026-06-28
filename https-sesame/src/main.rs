mod config;
mod state;
mod models;
mod auth;
mod proxy;
mod logging;

use axum::{
    routing::{get, post},
    middleware,
    Router,
};

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use tracing::info;

use config::EnvConfig;
use state::AppState;
use auth::knock_handler;
use proxy::proxy_dashboard;
use logging::{req_logger};

#[tokio::main]
async fn main() {
    let config = EnvConfig::load_env();

    logging::init();



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

    info!("Gateway listening on {}:{}", config.gateway_host, config.gateway_port);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

