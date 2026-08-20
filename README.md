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

## Maintenance update (2026): why it needed fixing

The original's neatest trick — reinterpreting `boxes.bin` as a slice of
`Option<WinCoords>`, where the `NonZeroU8` niche makes `None` (the frame
separator) free of cost — is **not portable across rustc versions**. The
compiler is allowed to reorder the fields of a `#[repr(packed)]` struct when it
applies the niche optimization, and modern rustc does: the in-memory layout
becomes `[w, h, x, y]` instead of the declared `[x, y, w, h]`. The result is
that every box is scrambled and any `x=0` box is misread as a frame separator,
so the video comes out as garbage. This repo now parses the bytes explicitly
(always correct, and still fast: ~300k 4-byte entries).

Two environment-related bugs were also fixed:

- The frame timeline used to be driven by the audio backend's clock
  (kira/cpal). On machines with virtual or remote audio devices that clock can
  be wildly unstable, freezing or randomly jumping the video. The timeline is
  now a plain wall clock at 30fps; audio plays best-effort, and the video
  keeps playing even if no audio device exists at all.
- `SWP_NOREDRAW` was supposed to keep windows' surfaces in sync cheaply. On
  modern Windows/DWM a window that is moved/resized without being redrawn
  keeps its **old** surface, so the picture froze into the previous frame's
  rectangles. Repaints are now always allowed — the fills are solid, the cost
  is negligible.

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
