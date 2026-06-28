use axum::{
    extract::{ConnectInfo, State},
    http::{StatusCode, HeaderMap},
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

use crate::models::KnockRequest;
use crate::state::AppState;
use crate::logging::get_client_ip;



pub async fn knock_handler(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<KnockRequest>
) -> impl IntoResponse
{
    let client_ip = get_client_ip(addr, &headers);
    let mut invalid = 0;

    {
        let mut map = state.failed_ips.write().unwrap();
        let mut used_nonces = state.used_nonces.write().unwrap();

        let info = map
        .entry(client_ip)
        .or_default();

        if let Some(expiry) = info.blocked_expiry {
            if Instant::now() < expiry {
                warn!("Blocked request! IP: {}", client_ip);
                invalid += 1;
                // add firewall rule here.
            }
        }
        if SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().abs_diff(payload.timestamp) > state.config.max_time_drift.into() {
            warn!("Invalid timestamp! IP = {}", client_ip);
            invalid += 1;
        }

        if used_nonces.contains_key(&payload.nonce) {
            warn!("Reused nonce! IP: {}", client_ip);
            invalid += 1;
        } else {
            used_nonces.insert(payload.nonce.clone(), Instant::now());
        }

    }

    if payload.passphrase != state.config.passphrase {
        warn!("Wrong password! IP: {}", client_ip);
        invalid += 1;
    }
    if invalid > 0{
        {
            let mut map = state.failed_ips.write().unwrap();

            let info = map
            .entry(client_ip)
            .or_default();

            info.attempts += 1;

            warn!("Attempts from {} = {}", client_ip, info.attempts);

            if info.attempts >= state.config.max_failed_attempts {
                info.blocked_expiry = Some(Instant::now() + Duration::from_secs(state.config.timeout_dur as u64 * (info.attempts - state.config.max_failed_attempts + 1) as u64));
            }
        }

        fake_failure().await;

        return StatusCode::NOT_FOUND;
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

    StatusCode::OK
}

pub async fn fake_failure() {
    let delay =
    rand::thread_rng().gen_range(1000..5000);

    tokio::time::sleep(
        Duration::from_millis(delay)
    ).await;
}
