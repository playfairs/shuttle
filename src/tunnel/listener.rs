use crate::error::Result;
use crate::transport::{create_server_endpoint, QuicConnection};
use std::net::SocketAddr;
use tracing::{debug, error};

pub async fn listen_for_tunnels(bind_addr: SocketAddr) -> Result<()> {
    let endpoint = create_server_endpoint(bind_addr).await?;
    println!("Tunnel server listening on {}", bind_addr);

    while let Some(incoming) = endpoint.accept().await {
        tokio::spawn(async move {
            if let Err(e) = handle_tunnel_accept(incoming.await).await {
                error!("Tunnel server error: {}", e);
            }
        });
    }

    Ok(())
}

async fn handle_tunnel_accept(
    connection_result: std::result::Result<quinn::Connection, quinn::ConnectionError>,
) -> Result<()> {
    let connection = connection_result
        .map_err(|e| crate::error::ShuttleError::ConnectionError(e.to_string()))?;
    let quic_conn = QuicConnection::new(connection);
    let remote_addr = quic_conn.remote_address();
    debug!("Tunnel connection from {}", remote_addr);

    while let Some(mut stream) = quic_conn.accept_stream().await? {
        let msg_data = stream.recv_message().await?.ok_or_else(|| {
            crate::error::ShuttleError::ProtocolError("No tunnel init".to_string())
        })?;

        let msg: crate::protocol::ProtocolMessage =
            crate::protocol::ProtocolMessage::decode(&msg_data)?;

        if !matches!(msg, crate::protocol::ProtocolMessage::TunnelInit) {
            return Err(crate::error::ShuttleError::ProtocolError(
                "Expected TunnelInit".to_string(),
            ));
        }

        tokio::spawn(async move {
            if let Err(e) = crate::tunnel::proxy::proxy_tunnel_stream(stream).await {
                error!("Tunnel stream error: {}", e);
            }
        });
    }

    Ok(())
}
