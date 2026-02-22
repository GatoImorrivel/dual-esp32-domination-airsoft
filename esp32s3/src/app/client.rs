use std::{sync::mpsc, sync::OnceLock, time::Duration};

use crate::{
    app::{AppEvent, AppState},
    game::MatchProgress,
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

    pub fn setup_wifi(&self, wifi_config: WifiConfig) -> anyhow::Result<()> {
        let (reply, rx) = mpsc::channel();
        self.tx.send(AppEvent::Configure { wifi_config, reply })?;
        let response = rx.recv_timeout(Duration::from_secs(5))??;

        Ok(response)
    }

    pub fn get_app_state(&self) -> anyhow::Result<AppState> {
        let (reply, rx) = mpsc::channel();
        self.tx.send(AppEvent::GetAppState { reply })?;
        let response = rx.recv_timeout(Duration::from_secs(5))?;

        Ok(response)
    }

    pub fn get() -> AppClient {
        APP_CLIENT
            .get()
            .expect("App client wasnt initialized yet")
            .clone()
    }
}
