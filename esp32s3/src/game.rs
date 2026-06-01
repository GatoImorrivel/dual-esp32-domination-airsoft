use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize)]
pub enum Team {
    Red,
    Blue,
}

#[derive(Debug, Clone, Copy)]
pub struct GameState {
    active: bool,
    current_team: Option<Team>,
    last_tick: Option<Instant>,
    team_red_time: Duration,
    team_blue_time: Duration,
    config: GameConfig,
}

impl Default for GameState {
    fn default() -> Self {
        GameState::new(GameConfig::default())
    }
}

impl GameState {
    pub fn new(config: GameConfig) -> Self {
        Self {
            active: false,
            current_team: None,
            last_tick: None,
            team_red_time: Duration::ZERO,
            team_blue_time: Duration::ZERO,
            config,
        }
    }

    pub fn current_config(&self) -> &GameConfig {
        &self.config
    }

    pub fn update_config(&mut self, new_config: GameConfig) {
        self.config = new_config
    }

    pub fn match_progress(&mut self) -> Option<MatchProgress> {
        Some(MatchProgress {
            scores: self.scores(),
            current_team: self.current_team(),
            is_active: self.active(),
            winner: self.winner(),
        })
    }

    pub fn active(&self) -> bool {
        self.active
    }

    /// Start or restart the game
    pub fn start(&mut self) {
        self.active = true;
        self.current_team = None;
        self.last_tick = Some(Instant::now());
        self.team_red_time = Duration::ZERO;
        self.team_blue_time = Duration::ZERO;
        log::info!("Game started");
    }

    /// Stop the game (no more accumulation)
    pub fn stop(&mut self) {
        self.active = false;
        self.current_team = None;
        self.last_tick = None;
        log::info!("Game stopped");
    }

    /// Handle a button press. Returns `true` if the game was active and ownership switched.
    pub fn button_press(&mut self, team: Team) -> bool {
        if !self.active {
            log::info!("{team:#?} pressed the button, ignoring due to game being inactive");
            return false;
        }

        // First, account for time so far
        self.tick();

        // Switch ownership
        self.current_team = Some(team);

        log::info!("{team:#?} pressed the button");
        true
    }

    /// Call this periodically (e.g. every 50–100 ms). Returns the winning team when the
    /// match ends by accumulated time on this tick.
    pub fn tick(&mut self) -> Option<Team> {
        if !self.active {
            return None;
        }

        let now = Instant::now();
        let Some(last) = self.last_tick else {
            self.last_tick = Some(now);
            return None;
        };

        let delta = now.duration_since(last);

        if let Some(owner) = self.current_team {
            match owner {
                Team::Blue => self.team_blue_time += delta,
                Team::Red => self.team_red_time += delta,
            }
        }

        self.last_tick = Some(now);

        let Some(winner) = self.winner() else {
            return None;
        };

        match winner {
            Team::Red => log::info!("Red team won"),
            Team::Blue => log::info!("Blue team won"),
        }
        self.stop();
        Some(winner)
    }

    /// Check if someone won
    pub fn winner(&self) -> Option<Team> {
        if self.team_blue_time >= self.config.blue_time_to_win {
            Some(Team::Blue)
        } else if self.team_red_time >= self.config.red_time_to_win {
            Some(Team::Red)
        } else {
            None
        }
    }

    /// Expose current scores (for UI / WS)
    pub fn scores(&self) -> Scores {
        Scores {
            red: self.team_red_time,
            blue: self.team_blue_time,
        }
    }

    /// Who currently owns the point
    pub fn current_team(&self) -> Option<Team> {
        self.current_team
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Scores {
    red: Duration,
    blue: Duration,
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchProgress {
    scores: Scores,
    is_active: bool,
    current_team: Option<Team>,
    winner: Option<Team>,
}

#[derive(Debug, Clone, Serialize, Copy, Deserialize)]
pub struct GameConfig {
    pub red_time_to_win: Duration,
    pub blue_time_to_win: Duration,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            blue_time_to_win: Duration::from_secs(10),
            red_time_to_win: Duration::from_secs(10),
        }
    }
}
