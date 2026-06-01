//! Team capture and winner cues — played on the ESP32 coprocessor by sound id.

use crate::{bt, game::Team};
use domination_uart::{SOUND_BLUE, SOUND_BLUE_WIN, SOUND_RED, SOUND_RED_WIN};

pub fn play_team(team: Team) {
    let sound_id = match team {
        Team::Red => SOUND_RED,
        Team::Blue => SOUND_BLUE,
    };
    queue_sound(sound_id, "team");
}

pub fn play_winner(team: Team) {
    let sound_id = match team {
        Team::Red => SOUND_RED_WIN,
        Team::Blue => SOUND_BLUE_WIN,
    };
    queue_sound(sound_id, "winner");
}

fn queue_sound(sound_id: u8, label: &str) {
    match bt::request_play_sound(sound_id) {
        Ok(id) => log::info!("queued {label} sound {sound_id} play_id={id}"),
        Err(e) => log::warn!("{label} audio failed: {e:#}"),
    }
}
