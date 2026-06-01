pub mod client;

use esp_idf_svc::hal::delay::FreeRtos;
use serde::Serialize;

use std::sync::mpsc;

use esp_idf_svc::nvs::EspDefaultNvsPartition;

use crate::{
    app::client::{AppClient, APP_CLIENT},
    audio,
    game::{GameConfig, GameState, MatchProgress},
    hardware::{
        wifi::{Wifi, WifiConfig, WifiNetwork},
        wifi_storage,
    },
};

type Reply<T> = mpsc::Sender<T>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AppState {
    Setup,
    Running,
}

#[derive(Debug)]
enum AppEvent {
    GetMatchProgress {
        reply: Reply<anyhow::Result<MatchProgress>>,
    },
    StartGame {
        reply: Reply<anyhow::Result<()>>,
    },
    StopGame {
        reply: Reply<anyhow::Result<()>>,
    },
    UpdateGameConfig {
        new_config: GameConfig,
        reply: Reply<anyhow::Result<()>>,
    },
    GetGameConfig {
        reply: Reply<anyhow::Result<GameConfig>>,
    },
    AppConfigure {
        wifi_config: WifiConfig,
        reply: Reply<anyhow::Result<()>>,
    },
    GetWifiConfig {
        reply: Reply<Option<WifiConfig>>,
    },
    GetAppState {
        reply: Reply<AppState>,
    },
    StartWifiScan {
        reply: Reply<anyhow::Result<()>>,
    },
    GetWifiScan {
        reply: Reply<WifiScanStatus>,
    },
}

#[derive(Debug, Clone)]
enum WifiScanState {
    Idle,
    Scanning,
    Ready(Vec<WifiNetwork>),
    Failed(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct WifiScanStatus {
    pub scanning: bool,
    pub networks: Vec<WifiNetwork>,
}

pub struct App {
    state: AppState,
    game: GameState,
    sender: mpsc::Sender<AppEvent>,
    receiver: mpsc::Receiver<AppEvent>,
    wifi: Wifi,
    wifi_scan: WifiScanState,
    nvs: EspDefaultNvsPartition,
}

impl App {
    pub fn new(wifi: Wifi, initial_state: AppState, nvs: EspDefaultNvsPartition) -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            state: initial_state,
            sender,
            receiver,
            game: GameState::default(),
            wifi,
            wifi_scan: WifiScanState::Idle,
            nvs,
        }
    }

    pub fn mut_game(&mut self) -> &mut GameState {
        &mut self.game
    }

    pub fn run<F: FnMut(&mut Self) -> anyhow::Result<()> + Send + 'static>(
        mut self,
        mut coroutine: F,
    ) {
        self.init_client();
        loop {
            if let Err(err) = coroutine(&mut self) {
                log::error!("Error in coroutine: {}", err);
            }

            if self.game.active() {
                if let Some(winner) = self.game.tick() {
                    audio::play_winner(winner);
                }
            }

            while let Ok(event) = self.receiver.try_recv() {
                if let Err(err) = self.handle_event(&event) {
                    log::error!("Failed to handle event {:?}", err);
                }
            }

            self.poll_wifi_scan();

            FreeRtos::delay_ms(20);
        }
    }

    fn handle_event(&mut self, event: &AppEvent) -> anyhow::Result<()> {
        match event {
            AppEvent::GetMatchProgress { reply } => {
                reply.send(Ok(self.game.match_progress().unwrap()))?;
            }
            AppEvent::StopGame { reply } => {
                if !self.game.active() {
                    reply.send(Err(anyhow::anyhow!("Jogo ainda não iniciado")))?;
                    return Ok(());
                }

                self.game.stop();
                reply.send(Ok(()))?;
            }
            AppEvent::StartGame { reply } => {
                if self.state == AppState::Setup {
                    reply.send(Err(anyhow::anyhow!(
                        "Aplicação está em modo de configuração"
                    )))?;
                    return Ok(());
                }

                if self.game.active() {
                    reply.send(Err(anyhow::anyhow!("Jogo já está iniciado")))?;
                    return Ok(());
                }

                self.game.start();
                reply.send(Ok(()))?;
            }
            AppEvent::GetGameConfig { reply } => {
                let config = self.game.current_config();
                reply.send(Ok(*config))?;
            }
            AppEvent::UpdateGameConfig { new_config, reply } => {
                self.game.update_config(*new_config);
                reply.send(Ok(()))?;
            }
            AppEvent::AppConfigure { wifi_config, reply } => {
                if let Err(e) = wifi_storage::save_wifi_config(&self.nvs, &wifi_config) {
                    log::error!("NVS save before configure failed: {e:#}");
                    reply.send(Err(e))?;
                    return Ok(());
                }
                let result =
                    esp_idf_svc::hal::task::block_on(self.wifi.configure(&wifi_config));
                match result {
                    Ok(()) => {
                        self.state = AppState::Running;
                        reply.send(Ok(()))?;
                    }
                    Err(e) => {
                        log::error!("Wi-Fi configure failed: {e:#}");
                        reply.send(Err(e))?;
                    }
                }
            }
            AppEvent::GetAppState { reply } => {
                reply.send(self.state)?;
            }
            AppEvent::GetWifiConfig { reply } => {
                let wifi_config = self.wifi.current_config();
                reply.send(wifi_config.clone())?;
            }
            AppEvent::StartWifiScan { reply } => {
                if self.state != AppState::Setup {
                    reply.send(Err(anyhow::anyhow!(
                        "Wi-Fi scan só está disponível durante a configuração inicial"
                    )))?;
                    return Ok(());
                }
                if matches!(self.wifi_scan, WifiScanState::Scanning) {
                    reply.send(Ok(()))?;
                    return Ok(());
                }
                match self.wifi.begin_scan() {
                    Ok(()) => {
                        self.wifi_scan = WifiScanState::Scanning;
                        reply.send(Ok(()))?;
                    }
                    Err(e) => {
                        self.wifi_scan = WifiScanState::Failed(e.to_string());
                        reply.send(Err(e))?;
                    }
                }
            }
            AppEvent::GetWifiScan { reply } => {
                self.poll_wifi_scan();
                reply.send(self.wifi_scan_status())?;
            }
        }
        Ok(())
    }

    fn poll_wifi_scan(&mut self) {
        if !matches!(self.wifi_scan, WifiScanState::Scanning) {
            return;
        }
        match self.wifi.poll_scan_result() {
            Ok(Some(networks)) => {
                self.wifi_scan = WifiScanState::Ready(networks);
            }
            Ok(None) => {}
            Err(e) => {
                log::error!("Wi-Fi scan poll failed: {e:#}");
                self.wifi_scan = WifiScanState::Failed(e.to_string());
            }
        }
    }

    fn wifi_scan_status(&self) -> WifiScanStatus {
        match &self.wifi_scan {
            WifiScanState::Idle => WifiScanStatus {
                scanning: false,
                networks: vec![],
            },
            WifiScanState::Scanning => WifiScanStatus {
                scanning: true,
                networks: vec![],
            },
            WifiScanState::Ready(networks) => WifiScanStatus {
                scanning: false,
                networks: networks.clone(),
            },
            WifiScanState::Failed(_) => WifiScanStatus {
                scanning: false,
                networks: vec![],
            },
        }
    }

    fn init_client(&self) {
        let client = AppClient {
            tx: self.sender.clone(),
        };

        APP_CLIENT.set(client).unwrap();
    }
}
