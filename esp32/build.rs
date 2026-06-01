//! ESP-IDF build + embedded team audio (MP3 → PCM via ffmpeg).

use std::path::{Path, PathBuf};
use std::process::Command;

/// Longest embedded clip (~2.6 s stereo 44.1 kHz) is ~449 KiB PCM.
const MAX_AUDIO_BYTES: u64 = 512 * 1024;

fn main() {
    let defaults = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("sdkconfig.defaults");
    std::env::set_var("SDKCONFIG_DEFAULTS", &defaults);
    println!("cargo:rerun-if-changed={}", defaults.display());

    convert_team_audio();

    embuild::espidf::sysenv::output();
}

fn convert_team_audio() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let audios_dir = manifest_dir.join("audios");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed={}", audios_dir.display());

    let assets = [
        ("vermelho-dominou.mp3", "red.pcm", "RED_PCM"),
        ("azul-dominou.mp3", "blue.pcm", "BLUE_PCM"),
        ("vencedor-vermelho.mp3", "red_win.pcm", "RED_WIN_PCM"),
        ("vencedor-azul.mp3", "blue_win.pcm", "BLUE_WIN_PCM"),
    ];

    for (mp3_name, pcm_name, env_key) in assets {
        let src = audios_dir.join(mp3_name);
        let dst = out_dir.join(pcm_name);
        println!("cargo:rerun-if-changed={}", src.display());
        convert_mp3_to_pcm(&src, &dst, mp3_name);
        println!("cargo:rustc-env={env_key}={}", dst.display());
    }
}

fn convert_mp3_to_pcm(src: &Path, dst: &Path, label: &str) {
    if !src.exists() {
        panic!(
            "missing team audio {} — add MP3 under esp32/audios/",
            src.display()
        );
    }

    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            src.to_str().unwrap(),
            "-f",
            "s16le",
            "-acodec",
            "pcm_s16le",
            "-ar",
            "44100",
            "-ac",
            "2",
            dst.to_str().unwrap(),
        ])
        .status()
        .unwrap_or_else(|e| panic!("ffmpeg required for {label}: {e}"));

    if !status.success() {
        panic!("ffmpeg failed for {label}");
    }

    let len = std::fs::metadata(dst).unwrap().len();
    if len > MAX_AUDIO_BYTES {
        panic!("PCM {label} is {len} bytes, exceeds {MAX_AUDIO_BYTES}");
    }
    if len % 4 != 0 {
        panic!("PCM {label} length {len} not stereo-aligned");
    }
    eprintln!("cargo:warning=embedded audio {label}: {len} bytes");
}
