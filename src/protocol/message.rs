use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u8 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolMessage {
    Hello { version: u8 },
    TransferInit { filename: String, size: u64 },
    TransferComplete { bytes_transferred: u64 },
    TransferError { reason: String },
    TunnelInit,
    TunnelClose,
}

impl ProtocolMessage {
    pub fn encode(&self) -> crate::error::Result<Vec<u8>> {
        Ok(bincode::serialize(self)?)
    }

    pub fn decode(data: &[u8]) -> crate::error::Result<Self> {
        Ok(bincode::deserialize(data)?)
    }
}
