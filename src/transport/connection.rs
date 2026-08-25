use crate::error::{Result, ShuttleError};
use bytes::BytesMut;
use quinn::{RecvStream, SendStream};
use std::net::SocketAddr;

pub struct QuicConnection {
    connection: quinn::Connection,
}

impl QuicConnection {
    pub fn new(connection: quinn::Connection) -> Self {
        QuicConnection { connection }
    }

    pub async fn open_stream(&self) -> Result<QuicStream> {
        let (send, recv) = self.connection.open_bi().await?;
        Ok(QuicStream { send, recv })
    }

    pub async fn accept_stream(&self) -> Result<Option<QuicStream>> {
        match self.connection.accept_bi().await {
            Ok((send, recv)) => Ok(Some(QuicStream { send, recv })),
            Err(quinn::ConnectionError::LocallyClosed) => Ok(None),
            Err(e) => Err(ShuttleError::ConnectionError(e.to_string())),
        }
    }

    pub fn remote_address(&self) -> SocketAddr {
        self.connection.remote_address()
    }

    pub async fn close(&self, code: u32, reason: &[u8]) {
        self.connection.close(code.into(), reason);
    }
}

pub struct QuicStream {
    send: SendStream,
    recv: RecvStream,
}

impl QuicStream {
    pub async fn send_message(&mut self, message: &[u8]) -> Result<()> {
        let frame = crate::protocol::framing::encode_message(message)?;
        self.send
            .write_all(&frame)
            .await
            .map_err(|e| ShuttleError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    pub async fn recv_message(&mut self) -> Result<Option<Vec<u8>>> {
        let mut buf = BytesMut::with_capacity(65536);
        let mut read_buf = vec![0u8; 4096];

        loop {
            match self.recv.read(&mut read_buf).await {
                Ok(Some(n)) => {
                    buf.extend_from_slice(&read_buf[..n]);
                    if let Some(msg) = crate::protocol::framing::decode_message(&mut buf)? {
                        return Ok(Some(msg));
                    }
                }
                Ok(None) => {
                    if buf.is_empty() {
                        return Ok(None);
                    }
                    return match crate::protocol::framing::decode_message(&mut buf)? {
                        Some(msg) => Ok(Some(msg)),
                        None => Err(ShuttleError::ProtocolError(
                            "Incomplete message".to_string(),
                        )),
                    };
                }
                Err(e) => return Err(ShuttleError::ConnectionError(e.to_string())),
            }
        }
    }

    pub async fn write_all(&mut self, data: &[u8]) -> Result<()> {
        self.send
            .write_all(data)
            .await
            .map_err(|e| ShuttleError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    pub async fn read(&mut self, buf: &mut [u8]) -> Result<Option<usize>> {
        match self.recv.read(buf).await {
            Ok(Some(n)) => Ok(Some(n)),
            Ok(None) => Ok(None),
            Err(e) => Err(ShuttleError::ConnectionError(e.to_string())),
        }
    }

    pub async fn finish(&mut self) -> Result<()> {
        self.send
            .finish()
            .map_err(|e| ShuttleError::ConnectionError(e.to_string()))?;
        Ok(())
    }
}
