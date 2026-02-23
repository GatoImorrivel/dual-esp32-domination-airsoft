use std::fmt::Debug;

use anyhow::Ok;
use esp_idf_svc::wifi::{AccessPointConfiguration, AsyncWifi, ClientConfiguration, EspWifi};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WifiConfig {
    APMode,
    ClientMode { ssid: String, password: String },
}

pub struct Wifi {
    wifi: AsyncWifi<EspWifi<'static>>,
    config: Option<WifiConfig>,
}

impl Debug for Wifi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Wifi")
    }
}

impl Wifi {
    pub fn init(wifi: AsyncWifi<EspWifi<'static>>) -> Self {
        Self { wifi, config: None }
    }

    pub fn current_config(&self) -> &Option<WifiConfig> {
        &self.config
    }

    pub async fn configure(&mut self, config: &WifiConfig) -> anyhow::Result<()> {
        match &config {
            WifiConfig::APMode => self.ap_mode().await?,
            WifiConfig::ClientMode { ssid, password } => self.client_mode(ssid, password).await?,
        }
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

        let config = esp_idf_svc::wifi::Configuration::Client(ClientConfiguration {
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

        Ok(())
    }

    pub async fn ap_mode(&mut self) -> anyhow::Result<()> {
        if let Some(mode) = &self.config {
            if *mode == WifiConfig::APMode {
                return Ok(());
            }
        }

        self.wifi.stop().await?;

        let config = esp_idf_svc::wifi::Configuration::AccessPoint(AccessPointConfiguration {
            ssid: "Dominacao".try_into().unwrap(),
            password: "sandidominacao".try_into().unwrap(),
            auth_method: esp_idf_svc::wifi::AuthMethod::WPA2Personal,
            ..Default::default()
        });

        self.wifi.set_configuration(&config)?;

        self.wifi.start().await?;

        self.config = Some(WifiConfig::APMode);

        Ok(())
    }
}
