mod config;
// mod state;
mod udp;
mod models;
mod auth;
mod logging;
mod database;

use axum::{
    routing::{get, post},
    middleware,
    Router,
};

use chrono::{Utc};

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use tracing::info;

use config::EnvConfig;
use models::{AppState, FailedIpInfo};
use auth::{knock_handler, authorise};
use logging::{req_logger};
use database::Database;
// use udp::start_udp_listener;

#[tokio::main]
async fn main() {
    let config = EnvConfig::load_env();

    logging::init();

    let mut authorised_ips = HashMap::new();
    let mut failed_ips = HashMap::new();
    let mut used_nonces = HashMap::new();

    let udp = true;
    let tcp = true;

    let database = if config.persistence {
        info!("Persistence enabled. Connecting to database...");
        let db = Database::connect(&config.db_conf)
            .await
            .expect("Failed to connect to database");

        db.init()
            .await
            .expect("Failed to initialise database");

        let now = Utc::now();
        for (ip, expires_at) in db.load_authorised_ips().await.expect("Failed to load authorised IPs") {
            if expires_at > now {
                authorised_ips.insert(ip, expires_at);
            }
        }
        for (ip, attempts, blocked_expiry) in db.load_failed_ips().await.expect("Failed to load blocked IPs") {
            if blocked_expiry > Some(now) {
                failed_ips.insert(ip, FailedIpInfo { attempts, blocked_expiry});
            }
        }
        for (nonce, used_at) in db.load_nonces().await.expect("Failed to load used nonces") {
            used_nonces.insert(nonce, used_at);
        }

        info!("Database connected!");
        Some(Arc::new(db))
    } else {
        info!("Persistence disabled");
        None
    };

    info!("Starting server...");

    let state = AppState {
        database,
        authorised_ips: Arc::new(RwLock::new(authorised_ips)),
        failed_ips: Arc::new(RwLock::new(failed_ips)),
        used_nonces: Arc::new(RwLock::new(used_nonces)),
        config: config.clone()
    };

    if udp {
        let sclone = state.clone();
        tokio::spawn(async move {
            if let Err(e) = udp::start_udp_listener(sclone).await {
                tracing::error!("UDP listener failed: {}", e);
            }
        });
    }
    if tcp {
        let app = Router::new()
        .route("/knock", post(knock_handler))
        .route("/authorise", get(authorise))
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
}

