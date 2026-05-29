use std::sync::{OnceLock, RwLock};

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
}

struct MockBtState {
    paired: Option<AudioSink>,
    discovered: Vec<AudioSink>,
}

static MOCK_BT: OnceLock<RwLock<MockBtState>> = OnceLock::new();

fn state() -> &'static RwLock<MockBtState> {
    MOCK_BT.get_or_init(|| {
        RwLock::new(MockBtState {
            paired: None,
            discovered: vec![],
        })
    })
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
    let guard = state()
        .read()
        .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
    Ok(BtSinksResponse {
        paired: guard.paired.clone(),
        discovered: guard.discovered.clone(),
    })
}

pub fn scan_sinks() -> anyhow::Result<BtSinksResponse> {
    let mut guard = state()
        .write()
        .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
    guard.discovered = mock_discovered_devices();
    log::info!(
        "BT mock scan: {} device(s) found",
        guard.discovered.len()
    );
    Ok(BtSinksResponse {
        paired: guard.paired.clone(),
        discovered: guard.discovered.clone(),
    })
}

pub fn pair_sink(address: &str) -> anyhow::Result<BtSinksResponse> {
    let mut guard = state()
        .write()
        .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;

    let sink = guard
        .discovered
        .iter()
        .find(|s| s.address.eq_ignore_ascii_case(address))
        .cloned()
        .or_else(|| {
            Some(AudioSink {
                address: address.to_string(),
                name: None,
            })
        })
        .ok_or_else(|| anyhow::anyhow!("Dispositivo nao encontrado"))?;

    log::info!("BT mock pair: {}", sink.address);
    guard.paired = Some(sink);

    Ok(BtSinksResponse {
        paired: guard.paired.clone(),
        discovered: guard.discovered.clone(),
    })
}

pub fn unpair_sink() -> anyhow::Result<BtSinksResponse> {
    let mut guard = state()
        .write()
        .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;

    if let Some(sink) = &guard.paired {
        log::info!("BT mock unpair: {}", sink.address);
    }
    guard.paired = None;

    Ok(BtSinksResponse {
        paired: None,
        discovered: guard.discovered.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reset() {
        let mut guard = state().write().unwrap();
        guard.paired = None;
        guard.discovered.clear();
    }

    #[test]
    fn scan_populates_discovered() {
        reset();
        let res = scan_sinks().unwrap();
        assert!(res.paired.is_none());
        assert_eq!(res.discovered.len(), 4);
    }

    #[test]
    fn pair_and_unpair() {
        reset();
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
        reset();
        let res = pair_sink("FF:FF:FF:FF:FF:FF").unwrap();
        assert_eq!(res.paired.as_ref().unwrap().address, "FF:FF:FF:FF:FF:FF");
    }
}
