//! Silent probe: cargo run --release --example audio_probe -- 12
//! Exercises the same playback path as the app without opening any windows.
use bad_apple::playback::Playback;

use kira::sound::streaming::StreamingSoundSettings;
use std::time::{Duration, Instant};

fn main() {
    let seconds: f64 = std::env::args()
        .nth(1)
        .map(|s| s.parse().expect("seconds"))
        .unwrap_or(12.0);
    assert!(seconds.is_finite() && seconds > 0.0);
    let launched = Instant::now();
    let mut playback = Playback::start(
        include_bytes!("../assets/bad apple.ogg"),
        StreamingSoundSettings::new().volume(0.0),
    );
    let queued = Instant::now();
    println!(
        "initialization_seconds={:.6}",
        launched.elapsed().as_secs_f64()
    );
    let mut last_video = 0.0;
    let mut last_log = 0.0;
    let mut max_error = 0.0_f64;
    let mut audio_advanced = false;
    while queued.elapsed().as_secs_f64() < seconds {
        let video = playback.position();
        let audio = playback.audio_position();
        assert!(video >= last_video, "video went backwards");
        last_video = video;
        if let Some(audio) = audio {
            let error = (audio - video).abs();
            max_error = max_error.max(error);
            assert!(error <= 1.0 / 30.0, "media cursor mismatch: {error}");
            audio_advanced |= audio > 0.0;
        }
        let wall = queued.elapsed().as_secs_f64();
        if wall - last_log >= 0.5 {
            println!(
                "wall={wall:.6} audio={audio:?} video={video:.6} frame={}",
                (video * 30.0) as u64
            );
            last_log = wall;
        }
        std::thread::sleep(Duration::from_millis(16));
    }
    println!("audio_advanced={audio_advanced} max_media_cursor_error_seconds={max_error:.6}");
    assert!(
        audio_advanced,
        "no working audio backend was available; sync was not verified"
    );
}
