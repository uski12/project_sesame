use axum::{
    extract::{ConnectInfo, Request},
    response::Response,
    middleware::Next,
};

use std::net::SocketAddr;

use tracing::{info, debug};

pub async fn req_logger(
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
