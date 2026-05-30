//! Bluetooth admin API (HTTP) and game audio via UART coprocessor.
//!
//! HTTP routes in `http/routes.rs` call `list_sinks`, `scan_sinks`, `pair_sink`, and
//! `unpair_sink` here. Production firmware (`#[cfg(not(test))]`) forwards every call to
//! [`dispatcher`] on UART1; [`mock`] exists only for unit tests under `#[cfg(test)]`.
//!
//! ## Wiring (921600 baud, `domination_uart::BAUD_RATE`)
//!
//! | S3 (main) | ESP32 (coprocessor) |
//! |-----------|---------------------|
//! | TX GPIO4  | RX GPIO16           |
//! | RX GPIO5  | TX GPIO17           |
//! | GND       | GND                 |
//!
//! ## Bring-up checklist (empty scan / no paired device)
//!
//! 1. Flash **both** `esp32s3` and `esp32` (Classic BT + A2DP sdkconfig on the coprocessor).
//! 2. Confirm cross-wiring above; boot log should show coprocessor reachability (see `check_coprocessor`).
//! 3. `POST /bt/scan` sets `scanning: true`; poll `GET /bt/sinks` until `scanning` is false (~10s).

#[cfg(not(test))]
mod dispatcher;

#[cfg(test)]
mod mock;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioSink {
    pub address: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtSinksResponse {
    pub paired: Option<AudioSink>,
    pub discovered: Vec<AudioSink>,
    #[serde(default)]
    pub scanning: bool,
    #[serde(default)]
    pub connected: bool,
}

#[cfg(not(test))]
pub fn init_dispatcher(uart: esp_idf_svc::hal::uart::UartDriver<'static>) -> anyhow::Result<()> {
    dispatcher::init(uart)
}

/// Ping the ESP32 coprocessor over UART (GetStatus). Call once after [`init_dispatcher`].
#[cfg(not(test))]
pub fn check_coprocessor() -> anyhow::Result<()> {
    dispatcher::refresh_status()
}

#[cfg(not(test))]
pub fn list_sinks() -> anyhow::Result<BtSinksResponse> {
    dispatcher::refresh_status()?;
    dispatcher::list_sinks_cached()
}

#[cfg(not(test))]
pub fn scan_sinks() -> anyhow::Result<BtSinksResponse> {
    dispatcher::start_scan()?;
    dispatcher::list_sinks_cached()
}

#[cfg(not(test))]
pub fn pair_sink(address: &str) -> anyhow::Result<BtSinksResponse> {
    dispatcher::pair_sink_dispatch(address)
}

#[cfg(not(test))]
pub fn unpair_sink() -> anyhow::Result<BtSinksResponse> {
    dispatcher::unpair_sink_dispatch()
}

#[cfg(not(test))]
pub fn request_play_sound(sound_id: u8) -> anyhow::Result<u32> {
    dispatcher::request_play_sound(sound_id)
}

#[cfg(test)]
pub use mock::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_populates_discovered() {
        mock::reset();
        let res = scan_sinks().unwrap();
        assert!(res.paired.is_none());
        assert_eq!(res.discovered.len(), 4);
        assert!(!res.scanning);
    }

    #[test]
    fn pair_and_unpair() {
        mock::reset();
        scan_sinks().unwrap();
        let paired = pair_sink("AA:BB:CC:11:22:33").unwrap();
        assert_eq!(
            paired.paired.as_ref().unwrap().name.as_deref(),
            Some("JBL Flip")
        );

        let unpaired = unpair_sink().unwrap();
        assert!(unpaired.paired.is_none());
    }

    #[test]
    fn pair_unknown_address_creates_sink() {
        mock::reset();
        let res = pair_sink("FF:FF:FF:FF:FF:FF").unwrap();
        assert_eq!(res.paired.as_ref().unwrap().address, "FF:FF:FF:FF:FF:FF");
    }
}
