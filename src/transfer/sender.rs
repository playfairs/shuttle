use crate::error::Result;
use crate::protocol::ProtocolMessage;
use crate::transfer::ProgressTracker;
use crate::transport::QuicStream;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tracing::{debug, info};

pub struct FileSender {
    stream: QuicStream,
}

impl FileSender {
    pub fn new(stream: QuicStream) -> Self {
        FileSender { stream }
    }

    pub async fn send_file(&mut self, file_path: &Path) -> Result<u64> {
        let metadata = tokio::fs::metadata(file_path).await?;
        let file_size = metadata.len();
        let filename = file_path
            .file_name()
            .ok_or_else(|| {
                crate::error::ShuttleError::FileNotFound("Invalid file path".to_string())
            })?
            .to_string_lossy()
            .to_string();

        info!("Sending file: {} ({} bytes)", filename, file_size);

        let init_msg = ProtocolMessage::TransferInit {
            filename: filename.clone(),
            size: file_size,
        };
        self.stream.send_message(&init_msg.encode()?).await?;

        let mut file = File::open(file_path).await?;
        let mut buffer = vec![0u8; 65536];
        let mut total_sent = 0u64;
        let progress = ProgressTracker::new(file_size);

        loop {
            let n = file.read(&mut buffer).await?;
            if n == 0 {
                break;
            }

            self.stream.write_all(&buffer[..n]).await?;
            total_sent += n as u64;
            progress.update(n as u64);

            if total_sent.is_multiple_of(1024 * 1024) {
                debug!(
                    "Sent {} / {} bytes ({:.1}%)",
                    progress.current(),
                    file_size,
                    progress.progress_percentage()
                );
            }
        }

        self.stream.finish().await?;

        if total_sent != file_size {
            return Err(crate::error::ShuttleError::TransferError(format!(
                "Expected to send {} bytes, but sent {}",
                file_size, total_sent
            )));
        }

        info!(
            "File transfer complete: {} bytes in {} seconds",
            total_sent,
            progress.elapsed_secs()
        );
        Ok(total_sent)
    }
}
