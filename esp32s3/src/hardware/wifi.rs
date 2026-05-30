use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;

use esp_idf_svc::handle::RawHandle;
use esp_idf_svc::wifi::{
    config::ScanConfig, AccessPointConfiguration, AccessPointInfo, AsyncWifi, AuthMethod,
    ClientConfiguration, Configuration, EspWifi,
};
use serde::{Deserialize, Serialize};

use crate::hardware::network;

const MAX_SCAN_RESULTS: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WifiConfig {
    APMode,
    ClientMode { ssid: String, password: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiNetwork {
    pub ssid: String,
    pub rssi: i8,
    pub auth: String,
    pub requires_password: bool,
}

pub struct Wifi {
    wifi: AsyncWifi<EspWifi<'static>>,
    config: Option<WifiConfig>,
    /// AP+STA already up (setup); scan must not call wifi.stop() or HTTP/mDNS drop.
    radio_ready_for_scan: bool,
}

impl Debug for Wifi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Wifi")
    }
}

impl Wifi {
    pub fn init(wifi: AsyncWifi<EspWifi<'static>>) -> Self {
        network::init_netifs(
            wifi.wifi().ap_netif().handle(),
            wifi.wifi().sta_netif().handle(),
        );
        Self {
            wifi,
            config: None,
            radio_ready_for_scan: false,
        }
    }

    pub fn current_config(&self) -> &Option<WifiConfig> {
        &self.config
    }

    pub async fn configure(&mut self, config: &WifiConfig) -> anyhow::Result<()> {
        match config {
            WifiConfig::APMode => self.ap_mode().await?,
            WifiConfig::ClientMode { ssid, password } => {
                self.client_mode(ssid, password).await?
            }
        }
        Ok(())
    }

    /// Start a non-blocking scan (setup Mixed mode; does not restart the radio).
    pub fn begin_scan(&mut self) -> anyhow::Result<()> {
        if !self.radio_ready_for_scan {
            anyhow::bail!("Wi-Fi scan requires setup mode (AP+STA)");
        }
        self.wifi
            .wifi_mut()
            .start_scan(&ScanConfig::default(), false)
            .map_err(|e| anyhow::anyhow!("start_scan: {e:?}"))?;
        log::info!("Wi-Fi scan started (non-blocking)");
        Ok(())
    }

    /// `None` while scanning; `Some` when complete or on error.
    pub fn poll_scan_result(&mut self) -> anyhow::Result<Option<Vec<WifiNetwork>>> {
        if !self.wifi.wifi().is_scan_done()? {
            return Ok(None);
        }
        let aps = self
            .wifi
            .wifi_mut()
            .get_scan_result()
            .map_err(|e| anyhow::anyhow!("get_scan_result: {e:?}"))?;
        let networks = dedupe_scan_results(aps);
        log::info!(
            "Wi-Fi scan done: {} network(s), mode={:?}",
            networks.len(),
            self.config
        );
        Ok(Some(networks))
    }

    /// Boot setup: Dominacao AP + STA for scan without tearing down the link later.
    pub async fn setup_mode(&mut self) -> anyhow::Result<()> {
        if self.radio_ready_for_scan {
            return Ok(());
        }

        if self.wifi.is_started()? {
            self.wifi.stop().await?;
        }

        let mixed = Configuration::Mixed(
            ClientConfiguration::default(),
            Self::default_ap_config(),
        );
        self.wifi.set_configuration(&mixed)?;
        self.wifi.start().await?;

        self.config = Some(WifiConfig::APMode);
        self.radio_ready_for_scan = true;
        network::set_softap_topology();
        log::info!("Wi-Fi setup: AP+STA (Dominacao), scan-ready without radio restart");
        Ok(())
    }

    pub async fn client_mode<S: AsRef<str>>(
        &mut self,
        new_ssid: S,
        password: S,
    ) -> anyhow::Result<()> {
        match &self.config {
            Some(mode) => match mode {
                WifiConfig::ClientMode { ssid, password: _ } => {
                    if ssid == new_ssid.as_ref() {
                        return Ok(());
                    }
                }
                _ => {}
            },
            _ => {}
        }

        self.wifi.stop().await?;

        let config = Configuration::Client(ClientConfiguration {
            ssid: new_ssid.as_ref().try_into().unwrap(),
            password: password.as_ref().try_into().unwrap(),
            ..Default::default()
        });

        self.wifi.set_configuration(&config)?;

        self.wifi.start().await?;

        self.wifi.connect().await?;

        self.wifi.wait_netif_up().await?;

        self.config = Some(WifiConfig::ClientMode {
            ssid: new_ssid.as_ref().to_owned(),
            password: password.as_ref().to_owned(),
        });
        self.radio_ready_for_scan = false;
        network::set_station_topology();

        Ok(())
    }

    /// Bounded STA connect for cold boot (uses driver connect/netif timeouts, ~30s total).
    pub async fn client_mode_with_timeout<S: AsRef<str>>(
        &mut self,
        new_ssid: S,
        password: S,
        _timeout: Duration,
    ) -> anyhow::Result<()> {
        self.client_mode(new_ssid, password).await
    }

    pub async fn ap_mode(&mut self) -> anyhow::Result<()> {
        if let Some(mode) = &self.config {
            if *mode == WifiConfig::APMode && !self.radio_ready_for_scan {
                return Ok(());
            }
        }

        self.wifi.stop().await?;

        let config = Configuration::AccessPoint(Self::default_ap_config());

        self.wifi.set_configuration(&config)?;

        self.wifi.start().await?;

        self.config = Some(WifiConfig::APMode);
        self.radio_ready_for_scan = false;
        network::set_softap_topology();

        Ok(())
    }

    fn default_ap_config() -> AccessPointConfiguration {
        AccessPointConfiguration {
            ssid: "Dominacao".try_into().unwrap(),
            password: "sandidominacao".try_into().unwrap(),
            auth_method: AuthMethod::WPA2Personal,
            ..Default::default()
        }
    }
}

pub(crate) fn dedupe_scan_results(aps: Vec<AccessPointInfo>) -> Vec<WifiNetwork> {
    let mut best: HashMap<String, WifiNetwork> = HashMap::new();

    for ap in aps {
        let ssid = ap.ssid.as_str().to_string();
        if ssid.is_empty() {
            continue;
        }
        let entry = ap_to_network(ap);
        best.entry(ssid)
            .and_modify(|existing| {
                if entry.rssi > existing.rssi {
                    *existing = entry.clone();
                }
            })
            .or_insert(entry);
    }

    let mut networks: Vec<WifiNetwork> = best.into_values().collect();
    networks.sort_by(|a, b| b.rssi.cmp(&a.rssi));
    networks.truncate(MAX_SCAN_RESULTS);
    networks
}

fn ap_to_network(ap: AccessPointInfo) -> WifiNetwork {
    let (auth, requires_password) = auth_label(ap.auth_method);
    WifiNetwork {
        ssid: ap.ssid.as_str().to_string(),
        rssi: ap.signal_strength,
        auth,
        requires_password,
    }
}

fn auth_label(method: Option<AuthMethod>) -> (String, bool) {
    match method {
        None | Some(AuthMethod::None) => ("Open".to_string(), false),
        Some(AuthMethod::WEP) => ("WEP".to_string(), true),
        Some(AuthMethod::WPA) => ("WPA".to_string(), true),
        Some(AuthMethod::WPA2Personal) => ("WPA2".to_string(), true),
        Some(AuthMethod::WPAWPA2Personal) => ("WPA/WPA2".to_string(), true),
        Some(AuthMethod::WPA2Enterprise) => ("WPA2-Enterprise".to_string(), true),
        Some(AuthMethod::WPA3Personal) => ("WPA3".to_string(), true),
        Some(AuthMethod::WPA2WPA3Personal) => ("WPA2/WPA3".to_string(), true),
        Some(AuthMethod::WAPIPersonal) => ("WAPI".to_string(), true),
    }
}
