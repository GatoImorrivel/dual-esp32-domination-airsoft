use std::sync::{OnceLock, RwLock};

use super::{AudioSink, BtSinksResponse};

struct MockBtState {
    paired: Option<AudioSink>,
    discovered: Vec<AudioSink>,
    scanning: bool,
}

static MOCK_BT: OnceLock<RwLock<MockBtState>> = OnceLock::new();

fn state() -> &'static RwLock<MockBtState> {
    MOCK_BT.get_or_init(|| {
        RwLock::new(MockBtState {
            paired: None,
            discovered: vec![],
            scanning: false,
        })
    })
}

pub fn reset() {
    let mut guard = state().write().unwrap();
    guard.paired = None;
    guard.discovered.clear();
    guard.scanning = false;
}

fn mock_discovered_devices() -> Vec<AudioSink> {
    vec![
        AudioSink {
            address: "AA:BB:CC:11:22:33".to_string(),
            name: Some("JBL Flip".to_string()),
        },
        AudioSink {
            address: "DD:EE:FF:44:55:66".to_string(),
            name: Some("Caixa Vermelha".to_string()),
        },
        AudioSink {
            address: "11:22:33:44:55:66".to_string(),
            name: Some("Speaker BT".to_string()),
        },
        AudioSink {
            address: "77:88:99:AA:BB:CC".to_string(),
            name: None,
        },
    ]
}

pub fn list_sinks() -> anyhow::Result<BtSinksResponse> {
    let guard = state().read().map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(BtSinksResponse {
        paired: guard.paired.clone(),
        discovered: guard.discovered.clone(),
        scanning: guard.scanning,
    })
}

pub fn scan_sinks() -> anyhow::Result<BtSinksResponse> {
    let mut guard = state().write().map_err(|e| anyhow::anyhow!("{e}"))?;
    guard.discovered = mock_discovered_devices();
    guard.scanning = false;
    Ok(BtSinksResponse {
        paired: guard.paired.clone(),
        discovered: guard.discovered.clone(),
        scanning: false,
    })
}

pub fn pair_sink(address: &str) -> anyhow::Result<BtSinksResponse> {
    let mut guard = state().write().map_err(|e| anyhow::anyhow!("{e}"))?;
    let sink = guard
        .discovered
        .iter()
        .find(|s| s.address.eq_ignore_ascii_case(address))
        .cloned()
        .unwrap_or(AudioSink {
            address: address.to_string(),
            name: None,
        });
    guard.paired = Some(sink);
    Ok(BtSinksResponse {
        paired: guard.paired.clone(),
        discovered: guard.discovered.clone(),
        scanning: guard.scanning,
    })
}

pub fn unpair_sink() -> anyhow::Result<BtSinksResponse> {
    let mut guard = state().write().map_err(|e| anyhow::anyhow!("{e}"))?;
    guard.paired = None;
    Ok(BtSinksResponse {
        paired: None,
        discovered: guard.discovered.clone(),
        scanning: guard.scanning,
    })
}

pub fn request_play_sound(_sound_id: u8) -> anyhow::Result<u32> {
    Ok(1)
}
