# Audio/video synchronization fix

Both projects previously selected video frames from a wall clock started
before the audio device was ready. The rewrite also included video setup time
in that clock. A silent probe on this machine measured about 1.23 seconds
between queuing the sound and its media position advancing. That explains a
fixed video lead from the opening, rather than a growing frame-rate error.

The renderer now prepares its windows before queuing audio and selects frame
`floor(sound_position_seconds * 30)`. A delayed audio start holds frame zero;
short audio stalls hold the matching video frame. Audio device and decode
failures use silent wall-clock playback starting at the last video position.
After five seconds without audio progress, the sound is stopped before that
fallback, so late audio cannot resume out of sync. The message loop waits for
input/timer events rather than spinning while idle.

Both visual styles now live in one executable: `--mode windows` uses adaptive
captions, and the default `--mode borderless` uses white tiles. Both provide
their own black backdrop and share the same playback clock.

The existing assets contain 6,572 video frames at 30 fps (219.066667 seconds)
and 219.094396 seconds of 48 kHz Vorbis audio. Their lengths differ by less
than one video frame. This fixes runtime timing; the original source video
would still be needed to verify any offset already baked into those assets,
or to compare a remaining perceptual/device-output delay.

## Verification

From the unified project folder:

```powershell
cargo test --locked --all-targets
cargo run --locked --release --example audio_probe -- 12
cargo build --locked --release --bins
```

The clock regression tests cover delayed startup, drift and short
stalls, unavailable audio, stalled-backend fallback, end/decode failure,
invalid positions, prevention of backward video time, and long audio/silent
pauses without watchdog fallback or a jump on resume.

`audio_probe` plays the existing audio at zero volume without opening windows.
It samples the production playback code approximately every 16 ms and checks
that video time is monotonic and within one video frame of the sound handle's
media position. It reports whether real audio progressed and the maximum
cursor difference. This does not measure speaker/Bluetooth latency or prove
the source assets match the original video.
