//! Team capture cues embedded at build time (`audios/*.mp3` → PCM in `build.rs`).

use domination_uart::{SOUND_BLUE, SOUND_RED};

static RED_PCM: &[u8] = include_bytes!(env!("RED_PCM"));
static BLUE_PCM: &[u8] = include_bytes!(env!("BLUE_PCM"));

pub fn is_valid_sound(sound_id: u8) -> bool {
    matches!(sound_id, SOUND_RED | SOUND_BLUE)
}

pub fn pcm_for_sound(sound_id: u8) -> Option<&'static [u8]> {
    match sound_id {
        SOUND_RED => Some(RED_PCM),
        SOUND_BLUE => Some(BLUE_PCM),
        _ => None,
    }
}
