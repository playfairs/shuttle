use crate::error::Result;
use bytes::{Buf, BytesMut};

const MAX_MESSAGE_SIZE: usize = 64 * 1024;

pub fn encode_message(message: &[u8]) -> Result<Vec<u8>> {
    if message.len() > MAX_MESSAGE_SIZE {
        return Err(crate::error::ShuttleError::ProtocolError(
            "Message too large".to_string(),
        ));
    }

    let len = message.len() as u32;
    let mut buf = Vec::with_capacity(4 + message.len());
    buf.extend_from_slice(&len.to_le_bytes());
    buf.extend_from_slice(message);
    Ok(buf)
}

pub fn decode_message(buf: &mut BytesMut) -> Result<Option<Vec<u8>>> {
    if buf.len() < 4 {
        return Ok(None);
    }

    let len_bytes = &buf[..4];
    let len = u32::from_le_bytes([len_bytes[0], len_bytes[1], len_bytes[2], len_bytes[3]]) as usize;

    if len > MAX_MESSAGE_SIZE {
        return Err(crate::error::ShuttleError::ProtocolError(
            "Message size exceeds limit".to_string(),
        ));
    }

    if buf.len() < 4 + len {
        return Ok(None);
    }

    buf.advance(4);
    let message = buf.split_to(len).to_vec();
    Ok(Some(message))
}
