use crate::error::Result;
use crate::transfer::FileSender;
use crate::transport::{create_client_endpoint, QuicConnection};
use std::net::SocketAddr;
use std::path::Path;
use tracing::info;

pub async fn handle_send(file_path: &Path, address: &str) -> Result<()> {
    if !file_path.exists() {
        return Err(crate::error::ShuttleError::FileNotFound(
            file_path.display().to_string(),
        ));
    }

    if !file_path.is_file() {
        return Err(crate::error::ShuttleError::InvalidConfig(
            "Path must be a file".to_string(),
        ));
    }

    let server_addr: SocketAddr = address.parse().map_err(|_| {
        crate::error::ShuttleError::InvalidConfig(format!("Invalid address: {}", address))
    })?;

    info!("Connecting to {} for transfer", server_addr);

    let (endpoint, _addr) = create_client_endpoint(server_addr).await?;

    let connection = endpoint
        .connect(server_addr, "localhost")
        .map_err(|e| crate::error::ShuttleError::ConnectionError(e.to_string()))?
        .await?;

    let quic_conn = QuicConnection::new(connection);

    let stream = quic_conn.open_stream().await?;
    let mut sender = FileSender::new(stream);
    sender.send_file(file_path).await?;

    quic_conn.close(0, b"transfer complete");

    info!("Transfer finished successfully");
    Ok(())
}
