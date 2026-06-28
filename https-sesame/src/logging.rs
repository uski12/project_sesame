use axum::{
    extract::{ConnectInfo, Request},
    http::HeaderMap,
    response::Response,
    middleware::Next,
};

use std::net::{
    IpAddr,
    SocketAddr,
};

use tracing::{info, debug};
use tracing_appender::rolling;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn get_client_ip(peer: SocketAddr, headers: &HeaderMap) -> IpAddr {
    if peer.ip().is_loopback() {
        if let Some(ip) = headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(",").next())
            .and_then(|s| s.trim().parse::<IpAddr>().ok())
            {
                return ip;
            }
    }
    peer.ip()
}

pub fn init() {
    let file = rolling::daily("logs", "gateway.log");

    tracing_subscriber::registry()
    .with(fmt::layer())
    .with(fmt::layer().with_writer(file).with_ansi(false))
    .init();
}


pub async fn req_logger(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Response {

    info!(
        ip = %get_client_ip(addr, &headers),
        method = %request.method(),
        uri = %request.uri(),
        "Request received: "
    );

    debug!(headers= ?request.headers());

    next.run(request).await
}
