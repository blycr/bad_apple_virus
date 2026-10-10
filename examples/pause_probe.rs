//! Silent integration check for the production audio pause/resume path.
use bad_apple::playback::Playback;
use kira::sound::streaming::StreamingSoundSettings;
use std::time::{Duration, Instant};

fn main() {
    let mut player = Playback::start(
        include_bytes!("../assets/bad apple.ogg"),
        StreamingSoundSettings::new().volume(0.0),
    );
    let start = Instant::now();
    while player.position() < 0.5 {
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "no audio progress"
        );
        std::thread::sleep(Duration::from_millis(16));
    }
    assert!(player.toggle_pause().expect("pause command"));
    let frozen = player.position();
    std::thread::sleep(Duration::from_millis(150));
    let audio = player.audio_position().expect("real audio is required");
    for _ in 0..60 {
        std::thread::sleep(Duration::from_millis(100));
        assert!(player.is_paused());
        assert_eq!(player.position(), frozen, "video advanced while paused");
        assert_eq!(
            player.audio_position(),
            Some(audio),
            "audio advanced while paused"
        );
    }
    assert!(!player.toggle_pause().expect("resume command"));
    let resumed = Instant::now();
    while player.position() < frozen + 0.5 {
        assert!(
            resumed.elapsed() < Duration::from_secs(2),
            "audio did not resume"
        );
        std::thread::sleep(Duration::from_millis(16));
    }
    let video = player.position();
    let audio = player
        .audio_position()
        .expect("audio fell back after a long pause");
    assert!(
        (audio - video).abs() <= 1.0 / 30.0,
        "lost sync after resume"
    );
    println!("PASS: six-second pause freezes audio/video; resume stays in sync");
}
