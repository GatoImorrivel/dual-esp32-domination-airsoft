use std::{sync::mpsc, sync::OnceLock, time::Duration};

use crate::{
    app::{AppEvent, AppState, WifiScanStatus},
    game::{GameConfig, MatchProgress},
    hardware::wifi::WifiConfig,
};

pub(super) static APP_CLIENT: OnceLock<AppClient> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct AppClient {
    pub(super) tx: mpsc::Sender<AppEvent>,
}

impl AppClient {
    pub fn get_match_progress(&self) -> anyhow::Result<MatchProgress> {
        let (reply, rx) = mpsc::channel();
        self.tx.send(AppEvent::GetMatchProgress { reply })?;
        let response = rx.recv_timeout(Duration::from_secs(5))??;

        Ok(response)
    }

    pub fn start_game(&self) -> anyhow::Result<()> {
        let (reply, rx) = mpsc::channel();
        self.tx.send(AppEvent::StartGame { reply })?;
        let response = rx.recv_timeout(Duration::from_secs(5))??;

        Ok(response)
    }

    pub fn stop_game(&self) -> anyhow::Result<()> {
        let (reply, rx) = mpsc::channel();
        self.tx.send(AppEvent::StopGame { reply })?;
        let response = rx.recv_timeout(Duration::from_secs(5))??;

        Ok(response)
    }

    pub fn get_game_config(&self) -> anyhow::Result<GameConfig> {
        let (reply, rx) = mpsc::channel();
        self.tx.send(AppEvent::GetGameConfig { reply })?;
        let response = rx.recv_timeout(Duration::from_secs(5))??;

        Ok(response)
    }

    pub fn update_game_config(&self, new_config: GameConfig) -> anyhow::Result<()> {
        let (reply, rx) = mpsc::channel();
        self.tx
            .send(AppEvent::UpdateGameConfig { new_config, reply })?;
        let response = rx.recv_timeout(Duration::from_secs(5))??;

        Ok(response)
    }

    pub fn setup_wifi(&self, wifi_config: WifiConfig) -> anyhow::Result<()> {
        let (reply, rx) = mpsc::channel();
        self.tx.send(AppEvent::AppConfigure { wifi_config, reply })?;
        let response = rx.recv_timeout(Duration::from_secs(90))??;

        Ok(response)
    }

    pub fn get_wifi_config(&self) -> anyhow::Result<Option<WifiConfig>> {
        let (reply, rx) = mpsc::channel();
        self.tx.send(AppEvent::GetWifiConfig { reply })?;
        let response = rx.recv_timeout(Duration::from_secs(5))?;

        Ok(response)
    }

    pub fn get_app_state(&self) -> anyhow::Result<AppState> {
        let (reply, rx) = mpsc::channel();
        self.tx.send(AppEvent::GetAppState { reply })?;
        let response = rx.recv_timeout(Duration::from_secs(5))?;

        Ok(response)
    }

    pub fn start_wifi_scan(&self) -> anyhow::Result<()> {
        let (reply, rx) = mpsc::channel();
        self.tx.send(AppEvent::StartWifiScan { reply })?;
        Ok(rx.recv_timeout(Duration::from_secs(5))??)
    }

    pub fn get_wifi_scan(&self) -> anyhow::Result<WifiScanStatus> {
        let (reply, rx) = mpsc::channel();
        self.tx.send(AppEvent::GetWifiScan { reply })?;
        Ok(rx.recv_timeout(Duration::from_secs(5))?)
    }

    pub fn get() -> AppClient {
        APP_CLIENT
            .get()
            .expect("App client wasnt initialized yet")
            .clone()
    }
}
