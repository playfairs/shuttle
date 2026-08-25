use crate::error::{Result, ShuttleError};
use crate::transfer::FileReceiver;
use crate::transport::{create_server_endpoint, QuicConnection};
use std::net::SocketAddr;
use std::path::Path;
use tracing::info;

pub async fn handle_receive(bind_addr: &str, output_dir: &Path) -> Result<()> {
    if !output_dir.exists() {
        return Err(ShuttleError::InvalidConfig(
            "Output directory does not exist".to_string(),
        ));
    }

    if !output_dir.is_dir() {
        return Err(ShuttleError::InvalidConfig(
            "Output path must be a directory".to_string(),
        ));
    }

    let addr: SocketAddr = bind_addr
        .parse()
        .map_err(|_| ShuttleError::InvalidConfig(format!("Invalid bind address: {}", bind_addr)))?;

    let endpoint = create_server_endpoint(addr).await?;
    info!("Listening for transfers on {}", addr);

    while let Some(connection) = endpoint.accept().await {
        let output_dir = output_dir.to_path_buf();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(connection.await, &output_dir).await {
                eprintln!("Connection error: {}", e);
            }
        });
    }

    Ok(())
}

async fn handle_connection(
    connection_result: std::result::Result<quinn::Connection, quinn::ConnectionError>,
    output_dir: &Path,
) -> Result<()> {
    let connection = connection_result.map_err(|e| ShuttleError::ConnectionError(e.to_string()))?;
    let quic_conn = QuicConnection::new(connection);

    while let Some(stream) = quic_conn.accept_stream().await? {
        let mut receiver = FileReceiver::new(stream);
        receiver.receive_file(output_dir).await?;
    }

    Ok(())
}
