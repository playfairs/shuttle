use crate::error::Result;
use std::net::SocketAddr;
use tracing::debug;

pub async fn handle_tunnel_client(local_addr: &str, remote_addr: &str) -> Result<()> {
    let local_socket: SocketAddr = local_addr.parse().map_err(|_| {
        crate::error::ShuttleError::InvalidConfig(format!("Invalid local address: {}", local_addr))
    })?;

    let remote_socket: SocketAddr = remote_addr.parse().map_err(|_| {
        crate::error::ShuttleError::InvalidConfig(format!(
            "Invalid remote address: {}",
            remote_addr
        ))
    })?;

    let listener = tokio::net::TcpListener::bind(&local_socket).await?;
    println!("Tunnel listening on {} -> {}", local_socket, remote_socket);

    loop {
        let (tcp_stream, peer_addr) = listener.accept().await?;
        debug!("Tunnel connection from {}", peer_addr);

        let remote = remote_socket;
        tokio::spawn(async move {
            if let Err(e) = crate::tunnel::proxy_tcp_through_quic(tcp_stream, remote).await {
                eprintln!("Tunnel connection error: {}", e);
            }
        });
    }
}
