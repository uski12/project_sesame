use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::{
    net::SocketAddr,
    time::Instant,
};
use tracing::{
    warn,
    error,
};

use crate::state::AppState;
use crate::auth::fake_failure;


pub async fn proxy_dashboard(
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
