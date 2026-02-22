pub mod client;

use esp_idf_svc::hal::delay::FreeRtos;
use serde::Serialize;

use std::sync::mpsc;

use crate::{
    app::client::{AppClient, APP_CLIENT},
    game::{GameState, MatchProgress},
    hardware::wifi::{Wifi, WifiConfig},
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
    Configure {
        wifi_config: WifiConfig,
        reply: Reply<anyhow::Result<()>>,
    },
    GetAppState {
        reply: Reply<AppState>,
    },
}

pub struct App {
    state: AppState,
    game: GameState,
    sender: mpsc::Sender<AppEvent>,
    receiver: mpsc::Receiver<AppEvent>,
    wifi: Wifi,
}

impl App {
    pub fn new(wifi: Wifi) -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            state: AppState::Setup,
            sender,
            receiver,
            game: GameState::default(),
            wifi,
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
                self.game.tick();
            }

            while let Ok(event) = self.receiver.try_recv() {
                if let Err(err) = self.handle_event(&event) {
                    log::error!("Failed to handle event {:?} {}", event, err);
                }
            }

            FreeRtos::delay_ms(20);
        }
    }

    fn handle_event(&mut self, event: &AppEvent) -> anyhow::Result<()> {
        match event {
            AppEvent::GetMatchProgress { reply } => {
                if !self.game.active() {
                    reply.send(Err(anyhow::anyhow!("Jogo ainda não iniciado")))?;
                    return Ok(());
                }

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
            AppEvent::Configure { wifi_config, reply } => {
                esp_idf_svc::hal::task::block_on(async {
                    self.wifi.configure(&wifi_config).await.unwrap();
                });
                reply.send(Ok(()))?;
                self.state = AppState::Running;
            }
            AppEvent::GetAppState { reply } => {
                reply.send(self.state)?;
            }
        }
        Ok(())
    }

    fn init_client(&self) {
        let client = AppClient {
            tx: self.sender.clone(),
        };

        APP_CLIENT.set(client).unwrap();
    }
}
