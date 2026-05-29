//! Team capture cues — played on the ESP32 coprocessor by sound id.

use crate::{bt, game::Team};
use domination_uart::{SOUND_BLUE, SOUND_RED};

pub fn play_team(team: Team) {
    let sound_id = match team {
        Team::Red => SOUND_RED,
        Team::Blue => SOUND_BLUE,
    };
    match bt::request_play_sound(sound_id) {
        Ok(id) => log::info!("queued team sound {sound_id} play_id={id}"),
        Err(e) => log::warn!("team audio failed: {e:#}"),
    }
}
