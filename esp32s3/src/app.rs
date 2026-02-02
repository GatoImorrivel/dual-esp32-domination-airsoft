use std::{
    sync::{mpsc, OnceLock},
    time::Duration,
};

use esp_idf_svc::hal::delay::FreeRtos;

use crate::{
    game::{GameState, MatchProgress},
    wifi::Wifi,
};

type Reply<T> = mpsc::Sender<T>;

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
}

pub struct App {
    game: GameState,
    sender: mpsc::Sender<AppEvent>,
    receiver: mpsc::Receiver<AppEvent>,
    wifi: Wifi,
}

impl App {
    pub fn new(wifi: Wifi) -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver,
            game: GameState::default(),
            wifi,
        }
    }

    pub fn run<F: Fn(&mut Self) -> anyhow::Result<()> + Send + 'static>(mut self, coroutine: F) {
        self.init_client();
        loop {
            if self.game.active() {
                self.game.tick();
            }

            while let Ok(event) = self.receiver.try_recv() {
                if let Err(err) = self.handle_event(&event) {
                    log::error!("Failed to handle event {:?} {}", event, err);
                }
            }

            if let Err(err) = coroutine(&mut self) {
                log::error!("Error in app coroutine {}", err);
                panic!();
            }

            FreeRtos::delay_ms(2000);
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
                if self.game.active() {
                    reply.send(Err(anyhow::anyhow!("Jogo já está iniciado")))?;
                    return Ok(());
                }

                self.game.start();
                reply.send(Ok(()))?;
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

static APP_CLIENT: OnceLock<AppClient> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct AppClient {
    tx: mpsc::Sender<AppEvent>,
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

    pub fn get() -> AppClient {
        APP_CLIENT
            .get()
            .expect("App client wasnt initialized yet")
            .clone()
    }
}
