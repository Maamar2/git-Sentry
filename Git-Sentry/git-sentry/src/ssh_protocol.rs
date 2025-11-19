use bytes::{Buf, BytesMut};
use std::io::Cursor;
use anyhow::{Result, anyhow};

/// SSH Agent Protocol message types
pub const SSH_AGENTC_REQUEST_IDENTITIES: u8 = 11;
pub const SSH_AGENTC_SIGN_REQUEST: u8 = 13;
pub const SSH_AGENTC_ADD_IDENTITY: u8 = 17;
pub const SSH_AGENTC_REMOVE_IDENTITY: u8 = 18;
pub const SSH_AGENTC_REMOVE_ALL_IDENTITIES: u8 = 19;
pub const SSH_AGENTC_ADD_ID_CONSTRAINED: u8 = 25;

pub const SSH_AGENT_IDENTITIES_ANSWER: u8 = 12;
pub const SSH_AGENT_SIGN_RESPONSE: u8 = 14;
pub const SSH_AGENT_FAILURE: u8 = 5;
pub const SSH_AGENT_SUCCESS: u8 = 6;

/// Parses SSH agent protocol message from bytes
pub fn parse_message(data: &[u8]) -> Result<SshMessage> {
    if data.len() < 5 {
        return Err(anyhow!("Message too short"));
    }

    let mut cursor = Cursor::new(data);
    let length = read_u32(&mut cursor)?;

    if data.len() < length as usize + 4 {
        return Err(anyhow!("Incomplete message"));
    }

    let msg_type = read_u8(&mut cursor)?;

    match msg_type {
        SSH_AGENTC_REQUEST_IDENTITIES => Ok(SshMessage::RequestIdentities),
        SSH_AGENTC_SIGN_REQUEST => {
            let key_blob = read_string(&mut cursor)?;
            let data = read_string(&mut cursor)?;
            let flags = read_u32(&mut cursor)?;
            Ok(SshMessage::SignRequest {
                key_blob,
                data,
                flags,
            })
        }
        SSH_AGENTC_REMOVE_ALL_IDENTITIES => Ok(SshMessage::RemoveAllIdentities),
        other => Ok(SshMessage::Unknown(other)),
    }
}

/// SSH Agent message types
#[derive(Clone, Debug)]
pub enum SshMessage {
    RequestIdentities,
    SignRequest {
        key_blob: Vec<u8>,
        data: Vec<u8>,
        flags: u32,
    },
    RemoveAllIdentities,
    Unknown(u8),
}

impl SshMessage {
    pub fn message_type(&self) -> u8 {
        match self {
            Self::RequestIdentities => SSH_AGENTC_REQUEST_IDENTITIES,
            Self::SignRequest { .. } => SSH_AGENTC_SIGN_REQUEST,
            Self::RemoveAllIdentities => SSH_AGENTC_REMOVE_ALL_IDENTITIES,
            Self::Unknown(t) => *t,
        }
    }

    pub fn is_sign_request(&self) -> bool {
        matches!(self, Self::SignRequest { .. })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(self.message_type());

        match self {
            Self::SignRequest {
                key_blob,
                data,
                flags,
            } => {
                write_string(&mut buf, key_blob);
                write_string(&mut buf, data);
                write_u32(&mut buf, *flags);
            }
            _ => {}
        }

        // Prepend length
        let mut msg = vec![0u8; 4];
        msg.extend(buf);
        let len = (msg.len() - 4) as u32;
        msg[0..4].copy_from_slice(&len.to_be_bytes());
        msg
    }
}

// Helper functions for reading/writing SSH protocol

fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8> {
    if cursor.remaining() < 1 {
        return Err(anyhow!("Not enough bytes"));
    }
    Ok(cursor.get_u8())
}

fn read_u32(cursor: &mut Cursor<&[u8]>) -> Result<u32> {
    if cursor.remaining() < 4 {
        return Err(anyhow!("Not enough bytes"));
    }
    Ok(cursor.get_u32())
}

fn read_string(cursor: &mut Cursor<&[u8]>) -> Result<Vec<u8>> {
    let len = read_u32(cursor)? as usize;
    if cursor.remaining() < len {
        return Err(anyhow!("Not enough bytes for string"));
    }
    let mut buf = vec![0u8; len];
    cursor.copy_to_slice(&mut buf);
    Ok(buf)
}

fn write_u8(buf: &mut Vec<u8>, val: u8) {
    buf.push(val);
}

fn write_u32(buf: &mut Vec<u8>, val: u32) {
    buf.extend_from_slice(&val.to_be_bytes());
}

fn write_string(buf: &mut Vec<u8>, data: &[u8]) {
    write_u32(buf, data.len() as u32);
    buf.extend_from_slice(data);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_request_identities() {
        let data = [0x00, 0x00, 0x00, 0x01, 0x0b]; // length=1, type=11
        let msg = parse_message(&data).unwrap();
        assert!(matches!(msg, SshMessage::RequestIdentities));
    }
}
