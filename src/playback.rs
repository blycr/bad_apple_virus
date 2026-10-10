//! Use the sound's media position, not time spent starting the process, for video.
use std::{
    io::Cursor,
    time::{Duration, Instant},
};

use kira::{
    manager::{backend::DefaultBackend, AudioManager, AudioManagerSettings},
    sound::{
        streaming::{StreamingSoundData, StreamingSoundHandle, StreamingSoundSettings},
        FromFileError, PlaybackState,
    },
    tween::Tween,
};

const AUDIO_STALL_TIMEOUT: Duration = Duration::from_secs(5);

pub struct Playback {
    // The manager must live as long as the sound.
    _manager: Option<AudioManager<DefaultBackend>>,
    sound: Option<StreamingSoundHandle<FromFileError>>,
    started: Instant,
    clock: PlaybackClock,
}

impl Playback {
    pub fn start(bytes: &'static [u8], settings: StreamingSoundSettings) -> Self {
        let mut sound = None;
        let manager = match AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()) {
            Ok(mut manager) => {
                match StreamingSoundData::from_cursor(Cursor::new(bytes), settings) {
                    Ok(data) => match manager.play(data) {
                        Ok(handle) => sound = Some(handle),
                        Err(error) => println!("audio: failed to play: {error:?} (video only)"),
                    },
                    Err(error) => println!("audio: failed to decode: {error:?} (video only)"),
                }
                Some(manager)
            }
            Err(error) => {
                println!("audio: no backend: {error:?} (video only)");
                None
            }
        };
        let clock = PlaybackClock::new(sound.is_some());
        Self {
            _manager: manager,
            sound,
            started: Instant::now(),
            clock,
        }
    }

    pub fn position(&mut self) -> f64 {
        let audio_position = self.sound.as_mut().and_then(|sound| {
            if let Some(error) = sound.pop_error() {
                println!("audio: decode error: {error:?} (video only)");
                None
            } else if sound.state() == PlaybackState::Stopped {
                // Continue any remaining video tail after the sound ends.
                None
            } else {
                Some(sound.position())
            }
        });
        let was_audio_active = self.clock.audio_active;
        let position = self.clock.sample(self.started.elapsed(), audio_position);
        if was_audio_active && !self.clock.audio_active {
            // A broken backend must not resume later and play out of sync.
            if let Some(mut sound) = self.sound.take() {
                let _ = sound.stop(Tween {
                    duration: Duration::ZERO,
                    ..Default::default()
                });
            }
            println!("audio: ended or stalled; continuing video from {position:.3}s");
        }
        position
    }

    pub fn is_paused(&self) -> bool {
        self.clock.paused
    }

    pub fn toggle_pause(&mut self) -> Result<bool, kira::CommandError> {
        let tween = Tween {
            duration: Duration::ZERO,
            ..Default::default()
        };
        if self.clock.paused {
            if let Some(sound) = &mut self.sound {
                sound.resume(tween)?;
            }
            self.clock.resume(self.started.elapsed());
        } else {
            self.position();
            if let Some(sound) = &mut self.sound {
                sound.pause(tween)?;
            }
            self.clock.paused = true;
        }
        println!(
            "playback: {}",
            if self.clock.paused {
                "paused"
            } else {
                "playing"
            }
        );
        Ok(self.clock.paused)
    }

    // Used by the headless synchronization probe.
    #[allow(dead_code)]
    pub fn audio_position(&self) -> Option<f64> {
        self.sound.as_ref().map(|sound| sound.position())
    }
}

struct PlaybackClock {
    audio_active: bool,
    position: f64,
    last_progress: Duration,
    wall_anchor: Duration,
    media_anchor: f64,
    paused: bool,
}

impl PlaybackClock {
    fn new(audio_active: bool) -> Self {
        Self {
            audio_active,
            position: 0.0,
            last_progress: Duration::ZERO,
            wall_anchor: Duration::ZERO,
            media_anchor: 0.0,
            paused: false,
        }
    }

    fn sample(&mut self, elapsed: Duration, audio_position: Option<f64>) -> f64 {
        if self.paused {
            return self.position;
        }
        if self.audio_active {
            if let Some(position) = audio_position.filter(|p| p.is_finite() && *p >= 0.0) {
                if position > self.position {
                    self.position = position;
                    self.last_progress = elapsed;
                }
                if elapsed.saturating_sub(self.last_progress) < AUDIO_STALL_TIMEOUT {
                    return self.position;
                }
            }
            // Start a continuous silent timeline from the last audio position.
            self.audio_active = false;
            self.wall_anchor = elapsed;
            self.media_anchor = self.position;
        }
        self.position = self.media_anchor + elapsed.saturating_sub(self.wall_anchor).as_secs_f64();
        self.position
    }

    fn resume(&mut self, elapsed: Duration) {
        self.paused = false;
        // Pausing is intentional, so it must not trigger the stall watchdog.
        self.last_progress = elapsed;
        self.wall_anchor = elapsed;
        self.media_anchor = self.position;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn seconds(s: f64) -> Duration {
        Duration::from_secs_f64(s)
    }

    #[test]
    fn silent_pause_excludes_paused_time_without_a_resume_jump() {
        let mut clock = PlaybackClock::new(false);
        assert_eq!(clock.sample(seconds(2.0), None), 2.0);
        clock.paused = true;
        assert_eq!(clock.sample(seconds(22.0), None), 2.0);
        clock.resume(seconds(22.0));
        assert_eq!(clock.sample(seconds(22.0), None), 2.0);
        assert_eq!(clock.sample(seconds(23.0), None), 3.0);
    }

    #[test]
    fn long_audio_pause_does_not_trigger_stall_fallback() {
        let mut clock = PlaybackClock::new(true);
        assert_eq!(clock.sample(seconds(2.0), Some(1.0)), 1.0);
        clock.paused = true;
        assert_eq!(clock.sample(seconds(22.0), Some(1.01)), 1.0);
        assert!(clock.audio_active);
        clock.resume(seconds(22.0));
        assert_eq!(clock.sample(seconds(22.1), Some(1.01)), 1.01);
        assert!(clock.audio_active);
        assert_eq!(clock.sample(seconds(23.0), Some(2.0)), 2.0);
    }

    #[test]
    fn delayed_audio_start_does_not_skip_the_opening() {
        let mut clock = PlaybackClock::new(true);
        assert_eq!(clock.sample(seconds(1.2), Some(0.0)), 0.0);
        assert_eq!(clock.sample(seconds(1.5), Some(0.25)), 0.25);
        assert_eq!((clock.sample(seconds(2.0), Some(0.75)) * 30.0) as usize, 22);
    }

    #[test]
    fn follows_audio_drift_and_short_stalls_instead_of_wall_time() {
        let mut clock = PlaybackClock::new(true);
        assert_eq!(clock.sample(seconds(1.0), Some(0.5)), 0.5);
        assert_eq!(clock.sample(seconds(2.0), Some(0.5)), 0.5);
        assert_eq!(clock.sample(seconds(3.0), Some(1.8)), 1.8);
        assert_eq!(clock.sample(seconds(180.0), Some(178.5)), 178.5);
    }

    #[test]
    fn no_audio_uses_wall_time_starting_at_zero() {
        let mut clock = PlaybackClock::new(false);
        assert_eq!(clock.sample(Duration::ZERO, None), 0.0);
        assert_eq!(clock.sample(seconds(2.5), None), 2.5);
    }

    #[test]
    fn stalled_backend_falls_back_without_a_jump_and_cannot_rejoin() {
        let mut clock = PlaybackClock::new(true);
        assert_eq!(clock.sample(seconds(1.0), Some(0.75)), 0.75);
        assert_eq!(clock.sample(seconds(6.0), Some(0.75)), 0.75);
        assert!(!clock.audio_active);
        assert_eq!(clock.sample(seconds(7.0), Some(4.0)), 1.75);
    }

    #[test]
    fn audio_end_or_decode_failure_preserves_the_remaining_video_tail() {
        let mut clock = PlaybackClock::new(true);
        assert_eq!(clock.sample(seconds(219.0), Some(218.8)), 218.8);
        assert_eq!(clock.sample(seconds(219.1), None), 218.8);
        assert!((clock.sample(seconds(219.4), None) - 219.1).abs() < 1e-9);
    }

    #[test]
    fn invalid_audio_position_cannot_poison_the_timeline() {
        let mut clock = PlaybackClock::new(true);
        assert_eq!(clock.sample(seconds(1.0), Some(f64::NAN)), 0.0);
        assert_eq!(clock.sample(seconds(2.0), None), 1.0);
    }

    #[test]
    fn minor_position_regression_does_not_rewind_video() {
        let mut clock = PlaybackClock::new(true);
        assert_eq!(clock.sample(seconds(1.0), Some(0.75)), 0.75);
        assert_eq!(clock.sample(seconds(1.1), Some(0.74)), 0.75);
    }
}
