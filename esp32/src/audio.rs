//! Team capture and winner cues embedded at build time (`audios/*.mp3` → PCM in `build.rs`).

use domination_uart::{SOUND_BLUE, SOUND_BLUE_WIN, SOUND_RED, SOUND_RED_WIN};

static RED_PCM: &[u8] = include_bytes!(env!("RED_PCM"));
static BLUE_PCM: &[u8] = include_bytes!(env!("BLUE_PCM"));
static RED_WIN_PCM: &[u8] = include_bytes!(env!("RED_WIN_PCM"));
static BLUE_WIN_PCM: &[u8] = include_bytes!(env!("BLUE_WIN_PCM"));

pub fn is_valid_sound(sound_id: u8) -> bool {
    matches!(
        sound_id,
        SOUND_RED | SOUND_BLUE | SOUND_RED_WIN | SOUND_BLUE_WIN
    )
}

pub fn pcm_for_sound(sound_id: u8) -> Option<&'static [u8]> {
    match sound_id {
        SOUND_RED => Some(RED_PCM),
        SOUND_BLUE => Some(BLUE_PCM),
        SOUND_RED_WIN => Some(RED_WIN_PCM),
        SOUND_BLUE_WIN => Some(BLUE_WIN_PCM),
        _ => None,
    }
}
