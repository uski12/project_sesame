use tokio::net::UdpSocket;

use tracing::{info, warn};

use crate::models::{ AppState, KnockRequest };
use crate::auth::knock_auth;
use crate::logging::get_client_ip;


pub async fn start_udp_listener(
    state: AppState,
) -> std::io::Result<()> {

    info!("Starting UDP listener...");
    let socket = UdpSocket::bind("0.0.0.0:8009").await?;

    info!("UDP socket successfully bound on {}", socket.local_addr()?);
    let mut buffer = [0u8; 4096];

    loop {
        let (len, addr) = socket.recv_from(&mut buffer).await?;

        let packet = &buffer[..len];

        let client_ip = get_client_ip(addr, None);
        info!("UDP packet received from {}", addr);

        let payload: Option<KnockRequest> =
        match serde_json::from_slice(packet) {
            Ok(payload) => {
                Some(payload)
            }
            Err(e) => {
                warn!("Invalid UDP payload from {}: {}", client_ip, e);
                None
            }
        };

        let _ = knock_auth(&state, client_ip, payload).await;
    }
}
