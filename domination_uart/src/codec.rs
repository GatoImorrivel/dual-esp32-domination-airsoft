use crate::protocol::Opcode;
use crate::{PROTOCOL_VERSION, SYNC};

/// Bytes after the 2-byte SYNC before payload: ver, flags, seq, opcode, len(2).
pub const HEADER_LEN: usize = 8;
pub const MIN_FRAME_LEN: usize = HEADER_LEN + 2; // + CRC

pub const FLAG_RESPONSE: u8 = 0x01;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub version: u8,
    pub is_response: bool,
    pub seq: u8,
    pub opcode: Opcode,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameDecodeError {
    NeedMoreBytes,
    BadSync,
    BadVersion,
    UnknownOpcode,
    LengthMismatch,
    CrcMismatch,
}

/// Encode a frame with empty or postcard-serialized payload.
pub fn encode_frame(
    opcode: Opcode,
    seq: u8,
    is_response: bool,
    payload: &[u8],
) -> Vec<u8> {
    let mut buf = Vec::with_capacity(MIN_FRAME_LEN + payload.len());
    buf.push(SYNC[0]);
    buf.push(SYNC[1]);
    buf.push(PROTOCOL_VERSION);
    let flags = if is_response { FLAG_RESPONSE } else { 0 };
    buf.push(flags);
    buf.push(seq);
    buf.push(opcode as u8);
    let len = payload.len().min(u16::MAX as usize) as u16;
    buf.push((len & 0xFF) as u8);
    buf.push((len >> 8) as u8);
    buf.extend_from_slice(&payload[..len as usize]);
    let crc = crc16_ccitt(&buf[2..]);
    buf.push((crc & 0xFF) as u8);
    buf.push((crc >> 8) as u8);
    buf
}

/// Incrementally decode frames from a receive buffer.
pub fn decode_frames(buf: &[u8]) -> Result<(Vec<Frame>, usize), FrameDecodeError> {
    let mut frames = Vec::new();
    let mut offset = 0;

    while offset + MIN_FRAME_LEN <= buf.len() {
        if buf[offset] != SYNC[0] || buf[offset + 1] != SYNC[1] {
            offset += 1;
            continue;
        }

        let version = buf[offset + 2];
        if version != PROTOCOL_VERSION {
            return Err(FrameDecodeError::BadVersion);
        }

        let flags = buf[offset + 3];
        let seq = buf[offset + 4];
        let opcode_byte = buf[offset + 5];
        let Some(opcode) = Opcode::from_u8(opcode_byte) else {
            return Err(FrameDecodeError::UnknownOpcode);
        };
        let len = u16::from_le_bytes([buf[offset + 6], buf[offset + 7]]) as usize;
        let frame_end = offset + HEADER_LEN + len + 2;
        if frame_end > buf.len() {
            return Ok((frames, offset));
        }

        let payload_start = offset + HEADER_LEN;
        let payload_end = payload_start + len;
        let crc_bytes = [buf[payload_end], buf[payload_end + 1]];
        let expected_crc = u16::from_le_bytes(crc_bytes);
        let actual_crc = crc16_ccitt(&buf[offset + 2..payload_end]);
        if expected_crc != actual_crc {
            return Err(FrameDecodeError::CrcMismatch);
        }

        frames.push(Frame {
            version,
            is_response: flags & FLAG_RESPONSE != 0,
            seq,
            opcode,
            payload: buf[payload_start..payload_end].to_vec(),
        });
        offset = frame_end;
    }

    Ok((frames, offset))
}

fn crc16_ccitt(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{Request, Response};
    use postcard;

    #[test]
    fn roundtrip_empty_ping() {
        let payload = postcard::to_allocvec(&Request::Ping).unwrap();
        let frame = encode_frame(Opcode::Ping, 1, false, &payload);
        let (frames, consumed) = decode_frames(&frame).unwrap();
        assert_eq!(consumed, frame.len());
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].opcode, Opcode::Ping);
        assert!(!frames[0].is_response);
    }

    #[test]
    fn crc_reject() {
        let mut frame = encode_frame(Opcode::Ping, 1, false, &[]);
        let last = frame.len() - 1;
        frame[last] ^= 0xFF;
        assert_eq!(decode_frames(&frame), Err(FrameDecodeError::CrcMismatch));
    }

    #[test]
    fn scan_result_roundtrip() {
        use crate::BtDevice;
        let resp = Response::ScanResult {
            devices: vec![BtDevice {
                name: Some("Speaker".into()),
                addr: [1, 2, 3, 4, 5, 6],
            }],
        };
        let payload = postcard::to_allocvec(&resp).unwrap();
        let frame = encode_frame(Opcode::Scan, 42, true, &payload);
        let (frames, _) = decode_frames(&frame).unwrap();
        let decoded: Response = postcard::from_bytes(&frames[0].payload).unwrap();
        match decoded {
            Response::ScanResult { devices } => assert_eq!(devices.len(), 1),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn partial_buffer_needs_more() {
        let frame = encode_frame(Opcode::Ping, 0, false, &[]);
        let (frames, consumed) = decode_frames(&frame[..frame.len() - 3]).unwrap();
        assert!(frames.is_empty());
        assert_eq!(consumed, 0);
    }
}
