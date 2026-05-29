use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Opcode {
    Ping = 0,
    Scan = 1,
    Connect = 2,
    Disconnect = 3,
    GetStatus = 4,
    PlayBegin = 5,
    PlayChunk = 6,
    PlayEnd = 7,
    PlayCancel = 8,
    /// Play embedded coprocessor clip by id (see [`crate::SOUND_RED`] / [`crate::SOUND_BLUE`]).
    PlaySound = 9,
}

impl Opcode {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Ping),
            1 => Some(Self::Scan),
            2 => Some(Self::Connect),
            3 => Some(Self::Disconnect),
            4 => Some(Self::GetStatus),
            5 => Some(Self::PlayBegin),
            6 => Some(Self::PlayChunk),
            7 => Some(Self::PlayEnd),
            8 => Some(Self::PlayCancel),
            9 => Some(Self::PlaySound),
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
    AudioTooLarge = 6,
    StalePlayId = 7,
    CrcMismatch = 8,
    InvalidPayload = 9,
    Internal = 10,
    UnknownSound = 11,
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
    PlayBegin { play_id: u32, total_len: u32 },
    PlayChunk {
        play_id: u32,
        offset: u32,
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },
    PlayEnd { play_id: u32 },
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

mod serde_bytes {
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error> {
        serde::Serialize::serialize(bytes.as_slice(), serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        let slice: Vec<u8> = serde::Deserialize::deserialize(deserializer)?;
        Ok(slice)
    }
}
