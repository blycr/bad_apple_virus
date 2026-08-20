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

## Why both projects needed the same two fixes

1. **The `Option<WinCoords>` niche trick is not portable.** The original
   reinterpreted the raw bytes of `boxes.bin` as `&[Option<WinCoords]>`,
   relying on the compiler keeping `#[repr(packed)]` fields in declaration
   order and storing the `None` discriminant in the last byte. Modern rustc
   reorders the fields for the niche optimization (layout becomes
   `[w, h, x, y]`), which scrambles every box and misreads `x=0` boxes as
   frame separators — the video renders as garbage. Both implementations now
   parse `boxes.bin` byte-by-byte: `(0,0,0,0)` separates frames, everything
   else is `(x, y, w, h)`. Always correct, still ~300k cheap entries.

2. **The audio clock is unreliable on this class of machine.** The original
   drove the 30fps frame timeline from kira/cpal's audio clock. On machines
   with virtual or remote audio devices (e.g. GameViewer/OrayIdd virtual
   display + audio drivers) that clock can freeze or jump randomly, so the
   video would crawl or skip. Both implementations now use a plain
   `std::time::Instant` wall clock for the timeline and play audio
   best-effort: the video stays smooth even if audio is missing, late, or
   broken. Audio initialization is also non-fatal (it can take seconds on
   virtual devices).

3. **`SWP_NOREDRAW` leaves stale window surfaces** (original only): on modern
   Windows/DWM a window moved/resized without a redraw keeps its old
   rectangles on screen, freezing the picture. Repaints are always allowed in
   both implementations; the fills are solid so the cost is negligible.

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
