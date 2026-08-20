# Bad Apple!! — windows-as-video, rewritten for modern Windows

This branch is a from-scratch reimplementation of the idea behind
[*Bad Apple but it's a Windows virus*](https://github.com/mon/bad_apple_virus)
(the original project lives on the `master` branch of this repository). It
borrows only the *concept* — render the 64×48 video grid as real, topmost
Windows windows, one window per rectangular "box" of each frame — and is built
to work reliably on a modern, high-DPI Windows machine with **any wallpaper**.

## What's different from the original

| | `master` (original) | `bad-apple-win` (this branch) |
|---|---|---|
| Black parts of the video | your desktop (needs a black wallpaper) | a full-screen black backdrop window (works on any wallpaper) |
| DPI | DPI-unaware (logical pixels, work area only) | PerMonitorV2-aware, physical pixels |
| Screen coverage | work area (taskbar stays visible) | full screen, including the taskbar area |
| Aspect ratio | stretched to the work area | preserved 4:3, centered with letterbox bars |
| Tiles | bordered windows (`WS_OVERLAPPEDWINDOW`, needs a FUDGE offset) | borderless solid tiles (`WS_POPUP`) |
| Quit | none (runs until the video ends) | press **Escape** |
| Frame timeline | wall clock at 30fps (see below) | wall clock at 30fps (see below) |
| `boxes.bin` parsing | explicit byte parsing (see below) | explicit byte parsing (see below) |

## Build & run

```powershell
cargo build --release
.\target\release\bad_apple_win.exe
```

The exe plays the full ~3:39 Bad Apple!! PV: 155 tile windows + a black
backdrop, at 30 fps. Press **Escape** to quit (or let it finish).

## Why the original rendered garbage — the four root causes

On modern toolchains and on machines like this one (Windows 10/11, 125% DPI,
virtual/remote audio drivers, non-black wallpaper), the original binary
rendered **no coherent picture at all** — no shapes, no video. Four
compounding root causes were found; both this rewrite and the `master` branch
fix them:

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
   compiler purely by luck of the layout. This rewrite parses `boxes.bin`
   byte-by-byte (`(0,0,0,0)` separates frames, everything else is
   `(x, y, w, h)`) — always correct, still ~300k cheap entries. The `master`
   branch instead keeps the trick and fixes the layout with `#[repr(C)]`.

2. **The audio clock is unreliable in virtual-audio environments.** The
   original drove the 30 fps timeline from kira/cpal's audio clock. On this
   machine (virtual audio device) that clock advances at random speeds — the
   video can sit on the first frames for 25+ seconds. Both implementations
   now drive the timeline from a wall clock (`std::time::Instant`); audio
   still plays, it just no longer clocks the video.

3. **Video start was delayed by audio initialization.** The virtual audio
   device takes 0–5 seconds (varying per run) to initialize, and the timeline
   used to start *after* it, so the video began late. Both implementations
   now take the wall-clock reference before audio initialization, and audio is
   best-effort — the video keeps playing even if the audio device is missing
   or broken.

4. **Windows kept stale surfaces instead of repainting.** `SWP_NOREDRAW` on a
   *resized* window keeps the window's stale old-size surface on modern
   Windows/DWM. In this rewrite the symptom was windows painting only their
   creation-size (1×1) "sliver" — the first show and the resize were applied
   in one batched `DeferWindowPos`, and the suppressed repaints left the rest
   of the window black. Fix: no `SWP_NOREDRAW` at all — resizes, moves and
   show/hide transitions all repaint (solid fills, negligible cost). The
   `master` branch keeps `SWP_NOREDRAW` for pure moves only.

Plus one display-fit fix in both: the source video is 4:3, and both
implementations preserve that aspect (single cell size, centered content with
letterbox bars) instead of stretching it to fill a 16:9 screen.

## How this implementation works

- `assets/boxes.bin` is the pre-processed video: for every frame, a list of
  `(x, y, w, h)` rectangles (in a 64×48 grid) that cover the white regions,
  produced by a greedy box-fitter (see `bad apple.py`). Frames are separated
  by `(0,0,0,0)`. It is embedded at compile time with
  `include_bytes_zstd!("assets/boxes.bin", 22)` (zstd level-22, decompressed
  at startup in a few ms).
- A full-screen black `WS_POPUP|WS_VISIBLE|WS_EX_TOPMOST` backdrop window is
  created first; the video's black parts are just the backdrop.
- 155 borderless white tile windows (`WS_POPUP`, class background brush
  `COLORREF(0x00FFFFFF)`) are created hidden, then per frame each tile is
  moved/sized/shown or hidden to match one box of the current frame.
- All window moves are batched with `BeginDeferWindowPos` /
  `DeferWindowPos` / `EndDeferWindowPos`, and dirty flags avoid touching
  unchanged windows — the same batching trick as the original.
- Geometry: with a PerMonitorV2 manifest the app sees physical pixels.
  `cell = min(scr_w / 64, scr_h / 48)`, content is `cell*64 × cell*48`,
  centered: `offset = ((scr_w - content_w) / 2, (scr_h - content_h) / 2)`.
- A `WM_TIMER` (16 ms) polls the wall clock and advances the timeline at
  exactly 30 fps, skipping missed frames instead of lagging behind.
- `RegisterHotKey` binds **Escape** to post `WM_QUIT`.

## Verification

The screenshots-vs-source-data pipeline used to validate this on the target
machine (1920×1080 @ 125 % DPI, RTX 3060, Windows 10/11):

```powershell
powershell -ExecutionPolicy Bypass -File .\capture_final.ps1   # captures at 6/12/20 s
python .\compare_final.py                                       # best-match against boxes.bin
```

On this machine the captures match the source frames essentially exactly
(12 s and 20 s: 0.000 mismatch; 6 s: 0.023 startup noise).

## Credits

- Original idea, `boxes.bin` pipeline and the batching technique: [mon / bad_apple_virus](https://github.com/mon/bad_apple_virus)
- This rewrite and the modern-toolchain fixes: blycr
