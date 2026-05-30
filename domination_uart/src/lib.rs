//! UART framing and postcard payloads for ESP32-S3 ↔ ESP32 BT coprocessor.
//!
//! Wiring (cross TX↔RX, common GND):
//! - ESP32-S3: UART1 TX=GPIO4, RX=GPIO5
//! - ESP32:     UART2 TX=GPIO17, RX=GPIO16
//! - Default baud: [`BAUD_RATE`] (921600)

pub mod codec;
pub mod mac;
pub mod protocol;

pub use codec::{decode_frames, encode_frame, Frame, FrameDecodeError};
pub use mac::{format_mac, parse_mac};
pub use protocol::*;

/// Default UART baud rate for the bridge link.
pub const BAUD_RATE: u32 = 921_600;

pub const SYNC: [u8; 2] = [0xD0, 0x6E];
pub const PROTOCOL_VERSION: u8 = 1;

/// Max PCM bytes per internal chunk when feeding A2DP from embedded clips.
pub const MAX_CHUNK: usize = 2048;
/// Max accumulated UART RX bytes before the bridge drops buffered data.
pub const MAX_RX_ACCUM: usize = 16 * 1024;
pub const MAX_DEVICES: usize = 32;

/// Embedded team cue ids on the ESP32 coprocessor (`esp32/audios/`).
pub const SOUND_RED: u8 = 0;
pub const SOUND_BLUE: u8 = 1;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BtDevice {
    pub name: Option<String>,
    pub addr: [u8; 6],
}
