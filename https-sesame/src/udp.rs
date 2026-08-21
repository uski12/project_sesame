use tokio::net::UdpSocket;

use tracing::{info, warn};

use crate::models::AppState;

pub async fn start_udp_listener(
    _state: AppState,
) -> std::io::Result<()> {
    info!("Starting UDP listener...");

    let socket = UdpSocket::bind("0.0.0.0:8009").await?;

    info!("UDP socket successfully bound on {}", socket.local_addr()?);

    let mut buffer = [0u8; 4096];

    loop {

        let (len, addr) = socket.recv_from(&mut buffer).await?;

        info!("UDP packet received from: {}, length: {}, text: {}", addr, len, String::from_utf8_lossy(&buffer[..len]));
    }
}
