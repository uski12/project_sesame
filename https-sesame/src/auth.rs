use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::{
    net::SocketAddr,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH}
};
use rand::Rng;
use tracing::{
    info,
    warn,
};

use crate::models::{KnockRequest, KnockResponse};
use crate::state::AppState;


pub async fn knock_handler(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<KnockRequest>
) -> impl IntoResponse {

    let client_ip = addr.ip();

    {
        let mut map = state.failed_ips.write().unwrap();
        let mut used_nonces = state.used_nonces.write().unwrap();

        let info = map
        .entry(client_ip)
        .or_default();

        if SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().abs_diff(payload.timestamp) > state.config.max_time_drift.into() {
            warn!("Invalid timestamp! IP = {}", client_ip);
            return (
                StatusCode::NOT_FOUND,
                Json(KnockResponse {
                    status: "not found".into(),
                })
            );
        }

        if used_nonces.contains_key(&payload.nonce) {
            warn!("Reused nonce! IP: {}", client_ip);
            return (
                StatusCode::NOT_FOUND,
                Json(KnockResponse {
                    status: "not found".into(),
                })
            );
        } else {
            used_nonces.insert(payload.nonce.clone(), Instant::now());
        }

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

pub async fn fake_failure() {
    let delay =
    rand::thread_rng().gen_range(1000..5000);

    tokio::time::sleep(
        Duration::from_millis(delay)
    ).await;
}
