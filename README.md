# Bad Apple but it's a Windows virus

A high-performance (I've never seen _anything_ like this run in realtime before)
rendition of Bad Apple, using Windows windows as the video surface.

It's not _actually_ a virus, but it is reminiscent of the viruses of old that
were more of a nuisance than anything.

Video demonstration:

[![Flandre Scarlet made out of windows](https://img.youtube.com/vi/EZpZwunMzuE/0.jpg)](https://www.youtube.com/watch?v=EZpZwunMzuE)

> **Heads-up**: this version draws the *white* parts of the video as windows
> over your desktop, so the desktop needs to be **black** (a solid black
> wallpaper) for the picture to read. If you can't change the wallpaper, or
> you run a high-DPI/remote setup, use the **`bad-apple-win`** branch instead:
> it is a self-contained rewrite that paints its own black backdrop and works
> on any wallpaper.

## Maintenance update (2026): why the picture was garbage, and what was fixed

On modern toolchains and on machines like this one (Windows 10/11, 125% DPI,
virtual/remote audio drivers, non-black wallpaper), the original binary
rendered **no coherent picture at all** — no shapes, no video. Four compounding
root causes were found:

1. **The `Option<WinCoords>` niche trick is not portable across rustc
   versions — the fatal one.** The original reinterpreted `boxes.bin` as a
   slice of `Option<WinCoords>`, relying on `#[repr(packed)]` keeping the
   fields in declaration order. The compiler is free to reorder fields when it
   applies the niche optimization, and modern rustc produces `[w, h, x, y]`
   instead of `[x, y, w, h]`. Measured behaviour: the bytes `[0,1,2,3]` — a
   perfectly valid box with `x=0` — decode as `None` (a frame separator), so
   every frame lost its left column and the whole box stream was misaligned;
   window sizes and positions became garbage (e.g. windows with `x` stuck at 1
   and zero-height rects). The original happened to work on the author's older
   compiler purely by luck of the layout. Fix: `#[repr(C)]` — a single
   attribute that fixes the layout in declaration order while keeping the
   4-byte `Option<WinCoords>` and the free `None` discriminant.

2. **The audio clock is unreliable in virtual-audio environments.** The frame
   timeline used to be driven by kira/cpal's audio clock. On this machine
   (virtual audio device) that clock advanced at random speeds — the video
   could sit on the first frames for 25+ seconds. Fix: drive the 30 fps
   timeline from a wall clock (`std::time::Instant`); audio still plays, it
   just no longer clocks the video.

3. **Video start was delayed by audio initialization.** The virtual audio
   device took 0–5 seconds (varying per run) to initialize, and the timeline
   was started *after* it, so the video began late. Fix: the wall-clock
   reference is taken before audio initialization, and audio is best-effort —
   the video keeps playing even if the audio device is missing or broken.

4. **Windows kept stale surfaces instead of repainting.** `SWP_NOREDRAW` on a
   *resized* window keeps the window's stale old-size surface on modern
   Windows/DWM, freezing the picture into the previous frame's rectangles (in
   the rewrite this showed up as windows painting only their creation-size
   "sliver"). Fix: `SWP_NOREDRAW` is now used only for pure moves (where it is
   still a real win); resizes and show/hide transitions repaint as usual. The
   fills are solid, so the extra repaints are cheap.

Plus one display-fit fix:

- The source video is 4:3. The original code stretched it to fill the work
  area (two independent per-axis ratios), which distorts the picture on 16:9
  screens. It now uses a single cell size (the smaller fit) and centers the
  content, preserving the 4:3 aspect with letterbox bars on wide screens.

## Why is it so performant?

- `DeferWindowPos` - even the most naive of projects can go from 1fps to 15fps
  by using this wonderful batched API instead of `SetWindowPos`.
- `WS_EX_TOOLWINDOW` to remove the taskbar entry
- Only showing/hiding/moving windows that actually need it (dirty tracking)
- Rust is *blazing fast*, don't you know?

## Could it be faster?

I suspect that choosing which windows to move/resize, such that each is resized
as little as possible, can increase performance - currently, the windows are just
used from largest to smallest, which can result in some location jitter as they
fit into different indexes.

## Future work

All of these I have done already in small tests, but they're both difficult to
make performant (copy dialogs are particularly slow), and difficult to arrange
into a pleasing display.

- Spawning `MessageBoxA` windows and taking their handle (thus avoiding the need
to replicate the layout of `MessageBoxA` for each version of Windows you run
on)
- Spawning Vista file copy dialogs using `IProgressDialog`
- Arranging windows in rolling sine waves, circles, etc

All of these I have not tried yet, but would be great additions:
- Water physics using hundreds of scroll bars
- Basic hard-body physics between windows
- Error noises synced with the audio (could just pre-render...)
- Notification bubbles
- Windows in the taskbar to show text (if the user has large taskbar buttons
  enabled)
- A large variety of error messages to delight the user with

## Building and such

Should be fine to just `cargo build --release`.

Look at `bad apple.py` for the pre-processing to take an input video and turn it
into `boxes.bin`, a space-optimized representation of the window bounds for each
frame. The script is jank, don't come complaining.

Check the PRs for some forks that improve the python script to be easier to run.

## Why's the repo archived?

I simply don't have the bandwidth to respond to questions and merge PRs for this little
project I threw out into the world. Feel free to fork it and make it do cool things!
