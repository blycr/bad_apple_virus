# Playback performance and controls

Default playback now uses the original 64x48 rectangles directly. The previous
256x192 refinement allocated 502 tile windows and increased repaint work;
it has been removed. Both modes of the unified executable use at most 155
tiles and one black backdrop. Video follows audio at the original 30 fps.

Resized, newly shown, or restyled visible windows still repaint immediately
to prevent black holes. Pure moves reuse existing surfaces. Shared edges map
to identical screen coordinates, and small blocks avoid caption minimum sizes.

## Controls

- **Ctrl+Alt+F8:** pause or resume both picture and sound.
- **Ctrl+Alt+F9:** exit and release the registered shortcuts.

These combinations work without window focus, suppress auto-repeat, and are
registered only while the program runs. Escape and unmodified function keys
are not registered. If a combination is already owned by another program,
startup shows a message and exits instead of overriding it.

Paused time is excluded from the silent fallback clock and from the audio
stall watchdog. Resuming keeps the same audio-driven timeline.

The unified project tests original geometry/window budget, shared screen
edges, long audio/silent pauses, and command-line mode selection.
`cargo run --release --example pause_probe`
checks a six-second pause against the real sound handle at zero volume.
