use crate::error::{Result, ShuttleError};
use crate::protocol::ProtocolMessage;
use crate::transport::QuicStream;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};

pub struct FileReceiver {
    stream: QuicStream,
}

impl FileReceiver {
    pub fn new(stream: QuicStream) -> Self {
        FileReceiver { stream }
    }

    pub async fn receive_file(&mut self, output_dir: &Path) -> Result<u64> {
        let msg_data =
            self.stream.recv_message().await?.ok_or_else(|| {
                ShuttleError::ProtocolError("No transfer init message".to_string())
            })?;

        let init_msg: ProtocolMessage = ProtocolMessage::decode(&msg_data)?;

        let (filename, expected_size) = match init_msg {
            ProtocolMessage::TransferInit { filename, size } => (filename, size),
            _ => {
                return Err(ShuttleError::ProtocolError(
                    "Expected TransferInit message".to_string(),
                ))
            }
        };

        let output_path = output_dir.join(&filename);

        if output_path.exists() {
            return Err(ShuttleError::FileExists(output_path.display().to_string()));
        }

        info!("Receiving file: {} ({} bytes)", filename, expected_size);

        let mut file = File::create(&output_path).await?;
        let mut buffer = vec![0u8; 65536];
        let mut total_received = 0u64;

        loop {
            match self.stream.read(&mut buffer).await? {
                Some(0) | None => break,
                Some(n) => {
                    file.write_all(&buffer[..n]).await?;
                    total_received += n as u64;

                    if total_received % (1024 * 1024) == 0 {
                        debug!("Received {} / {} bytes", total_received, expected_size);
                    }
                }
            }
        }

        file.sync_all().await?;
        drop(file);

        if total_received != expected_size {
            tokio::fs::remove_file(&output_path).await?;
            return Err(ShuttleError::TransferError(format!(
                "Expected {} bytes, but received {}",
                expected_size, total_received
            )));
        }

        info!(
            "File transfer complete: {} bytes to {}",
            total_received,
            output_path.display()
        );
        Ok(total_received)
    }
}
