use axum::{
    extract::{ConnectInfo, State, rejection::JsonRejection},
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
    Json,
};
use std::{
    net::SocketAddr,
    time::{Duration},
};

use chrono::{Duration as ChronoDuration, Utc};
use rand::Rng;
use tracing::{
    info,
    warn,
};

use crate::models::KnockRequest;
use crate::models::AppState;
use crate::logging::get_client_ip;



pub async fn knock_handler(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    payload: Result<Json<KnockRequest>, JsonRejection>
) -> impl IntoResponse
{
    let client_ip = get_client_ip(addr, &headers);

    let Json(payload) = match payload {
        Ok(payload) => payload,
        Err(_) => {
            {
                let mut map = state.failed_ips.write().unwrap();

                let info = map
                .entry(client_ip)
                .or_default();

                info.attempts += 1;

                warn!("Invalid knock payload! IP: {}, attempts: {}", client_ip, info.attempts);

                if info.attempts >= state.config.max_failed_attempts {
                    info.blocked_expiry = Some(Utc::now() + ChronoDuration::seconds(state.config.timeout_dur as i64 * (info.attempts - state.config.max_failed_attempts + 1) as i64));

                     // add firewall blocking here
                }
            }

            fake_failure().await;
            return StatusCode::REQUEST_TIMEOUT;
        }
    };

    let mut invalid = 0;
    {
        let mut map = state.failed_ips.write().unwrap();
        let mut used_nonces = state.used_nonces.write().unwrap();

        let info = map
        .entry(client_ip)
        .or_default();

        if let Some(expiry) = info.blocked_expiry {
            if Utc::now() < expiry {
                warn!("Blocked request! IP: {}", client_ip);
                invalid += 1;
                // add firewall rule here.
            }
        }

        //validate timestamp
        if (Utc::now() - payload.timestamp).num_seconds().unsigned_abs() > state.config.max_time_drift.into() {
            warn!("Invalid timestamp! IP = {}", client_ip);
            invalid += 1;
        }

        if used_nonces.contains_key(&payload.nonce) {
            warn!("Reused nonce! IP: {}", client_ip);
            invalid += 1;
        } else {
            used_nonces.insert(payload.nonce.clone(), Utc::now());
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
                info.blocked_expiry = Some(Utc::now() + ChronoDuration::seconds(state.config.timeout_dur as i64 * (info.attempts - state.config.max_failed_attempts + 1) as i64));
                // add firewall blocking here
            }
        }

        fake_failure().await;

        return StatusCode::REQUEST_TIMEOUT;
    }

    state
    .failed_ips.write()
    .unwrap()
    .remove(&client_ip);

    let auth_expiry = Utc::now() + ChronoDuration::seconds(state.config.auth_dur as i64);

    state
    .authorised_ips
    .write()
    .unwrap()
    .insert(client_ip, auth_expiry);

    info!("Authorised {}", client_ip);

    fake_failure().await;

    StatusCode::GATEWAY_TIMEOUT
}

pub async fn authorise(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap
) -> StatusCode {

    let client_ip = get_client_ip(addr, &headers);

    let authorised = state
        .authorised_ips
        .read()
        .unwrap()
        .get(&client_ip)
        .is_some_and(|expiry| *expiry > Utc::now());

    if authorised {
        StatusCode::OK
    } else {
        warn!("Unauthorised request! IP: {}", client_ip);
        StatusCode::NOT_FOUND
    }
}



pub async fn fake_failure() {
    let delay =
    rand::thread_rng().gen_range(1000..5000);

    tokio::time::sleep(
        Duration::from_millis(delay)
    ).await;
}
