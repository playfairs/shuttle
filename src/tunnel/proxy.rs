use crate::error::Result;
use crate::protocol::ProtocolMessage;
use crate::transport::{create_client_endpoint, QuicConnection};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::debug;

pub async fn proxy_tcp_through_quic(
    mut tcp_stream: tokio::net::TcpStream,
    remote_addr: SocketAddr,
) -> Result<()> {
    let (endpoint, _) = create_client_endpoint(remote_addr).await?;

    let connection = endpoint
        .connect(remote_addr, "localhost")
        .map_err(|e| crate::error::ShuttleError::ConnectionError(e.to_string()))?
        .await?;

    let quic_conn = QuicConnection::new(connection);
    let mut quic_stream = quic_conn.open_stream().await?;

    let init_msg = ProtocolMessage::TunnelInit;
    quic_stream.send_message(&init_msg.encode()?).await?;

    let (mut tcp_read, mut tcp_write) = tcp_stream.split();
    let mut tcp_buf = vec![0u8; 65536];
    let mut quic_buf = vec![0u8; 65536];

    loop {
        tokio::select! {
            result = tcp_read.read(&mut tcp_buf) => {
                match result? {
                    0 => {
                        quic_stream.finish().await?;
                        break;
                    }
                    n => {
                        quic_stream.write_all(&tcp_buf[..n]).await?;
                    }
                }
            }
            result = quic_stream.read(&mut quic_buf) => {
                match result? {
                    None | Some(0) => {
                        tcp_write.shutdown().await?;
                        break;
                    }
                    Some(n) => {
                        tcp_write.write_all(&quic_buf[..n]).await?;
                    }
                }
            }
        }
    }

    Ok(())
}

pub async fn proxy_tunnel_stream(mut quic_stream: crate::transport::QuicStream) -> Result<()> {
    let mut buf = vec![0u8; 65536];
    debug!("Proxying tunnel stream");

    while let Some(n) = quic_stream.read(&mut buf).await? {
        if n == 0 {
            break;
        }
    }

    Ok(())
}
