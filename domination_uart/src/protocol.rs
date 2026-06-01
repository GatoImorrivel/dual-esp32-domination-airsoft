use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Opcode {
    Ping = 0,
    Scan = 1,
    Connect = 2,
    Disconnect = 3,
    GetStatus = 4,
    PlayCancel = 5,
    /// Play embedded coprocessor clip by id (see [`crate::SOUND_RED`], [`crate::SOUND_BLUE`],
    /// [`crate::SOUND_RED_WIN`], [`crate::SOUND_BLUE_WIN`]).
    PlaySound = 6,
}

impl Opcode {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Ping),
            1 => Some(Self::Scan),
            2 => Some(Self::Connect),
            3 => Some(Self::Disconnect),
            4 => Some(Self::GetStatus),
            5 => Some(Self::PlayCancel),
            6 => Some(Self::PlaySound),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u16)]
pub enum ErrorCode {
    Busy = 1,
    NotConnected = 2,
    InvalidAddr = 3,
    ScanFailed = 4,
    ConnectFailed = 5,
    CrcMismatch = 6,
    InvalidPayload = 7,
    Internal = 8,
    UnknownSound = 9,
}

/// Request body encoded as postcard inside a frame payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Ping,
    Scan { duration_secs: u8 },
    Connect {
        addr: [u8; 6],
        name: Option<String>,
    },
    Disconnect,
    GetStatus,
    PlayCancel { play_id: u32 },
    PlaySound { play_id: u32, sound_id: u8 },
}

/// Response body encoded as postcard inside a frame payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Pong,
    Ok,
    Error { code: ErrorCode },
    ScanResult { devices: Vec<super::BtDevice> },
    Status {
        paired: Option<super::BtDevice>,
        connected: bool,
    },
}
