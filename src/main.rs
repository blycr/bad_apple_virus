//! Bad Apple, rendered with real Windows windows as the display surface.
//!
//! Concept borrowed from mon's `bad_apple_virus`, rewritten from scratch and
//! adapted to this machine:
//!   - explicit full-screen black backdrop (works on any wallpaper)
//!   - PerMonitorV2 DPI awareness (physical pixels, no DWM blur at 125%)
//!   - integer cell size => borderless tiles tile without seams
//!   - 4:3 letterboxed and centred on the full screen
//!   - Escape (registered hotkey) quits cleanly

#![windows_subsystem = "windows"]

use std::io::Cursor;
use std::num::NonZeroU8;
use std::time::Instant;

use include_bytes_zstd::include_bytes_zstd;
use kira::manager::{backend::DefaultBackend, AudioManager, AudioManagerSettings};
use kira::sound::streaming::{StreamingSoundData, StreamingSoundSettings};
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::CreateSolidBrush;
use windows::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, VK_ESCAPE};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::*;

const TILE_CLASS: &str = "BadAppleTile\0";
const BG_CLASS: &str = "BadAppleBG\0";
const HOTKEY_ID: i32 = 1;

/// Enough tiles to draw the densest frame of boxes.bin.
const MAX_WINDOWS: usize = 155;
/// The video grid is 64x48 (4:3).
const BASE_WIDTH: u8 = 64;
const BASE_HEIGHT: u8 = 48;

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    _lparam: LPARAM,
) -> LRESULT {
    match msg {
        // Escape was pressed (or the backdrop was closed): exit the app.
        WM_HOTKEY if wparam.0 as i32 == HOTKEY_ID => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        WM_CLOSE => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcA(hwnd, msg, wparam, _lparam),
    }
}

fn register_classes() -> Result<HINSTANCE> {
    let hinst = unsafe { GetModuleHandleW(PCWSTR::null()) }?;
    let white = unsafe { CreateSolidBrush(COLORREF(0x00FF_FFFF)) };
    let black = unsafe { CreateSolidBrush(COLORREF(0)) };

    unsafe {
        let tile_wc = WNDCLASSA {
            lpfnWndProc: Some(wnd_proc),
            hInstance: hinst,
            lpszClassName: PCSTR(TILE_CLASS.as_ptr() as _),
            hbrBackground: white,
            ..Default::default()
        };
        RegisterClassA(&tile_wc);

        let bg_wc = WNDCLASSA {
            lpfnWndProc: Some(wnd_proc),
            hInstance: hinst,
            lpszClassName: PCSTR(BG_CLASS.as_ptr() as _),
            hbrBackground: black,
            ..Default::default()
        };
        RegisterClassA(&bg_wc);
    }

    Ok(hinst)
}

/// One borderless window == one rectangular white "pixel" of the video.
/// Tracks dirty state so a Win32 call is only issued when something changed.
struct TileWindow {
    hwnd: HWND,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    pos_dirty: bool,
    size_dirty: bool,
    visible: bool,
    vis_dirty: bool,
}

impl TileWindow {
    fn create(hinst: HINSTANCE) -> Self {
        let hwnd = unsafe {
            CreateWindowExA(
                WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
                PCSTR(TILE_CLASS.as_ptr() as _),
                s!("Bad Apple!!"),
                WS_POPUP,
                0,
                0,
                1,
                1,
                None,
                None,
                hinst,
                None,
            )
        };
        assert!(hwnd.0 != 0);
        Self {
            hwnd,
            x: 0,
            y: 0,
            w: 0,
            h: 0,
            pos_dirty: false,
            size_dirty: false,
            visible: false,
            vis_dirty: false,
        }
    }

    fn set_rect(&mut self, x: i32, y: i32, w: i32, h: i32) {
        self.pos_dirty = self.x != x || self.y != y;
        self.size_dirty = self.w != w || self.h != h;
        self.x = x;
        self.y = y;
        self.w = w;
        self.h = h;
    }

    fn set_visible(&mut self, v: bool) {
        self.vis_dirty = self.visible != v;
        self.visible = v;
    }

    fn dirty(&self) -> bool {
        self.pos_dirty || self.size_dirty || self.vis_dirty
    }

    fn apply(&mut self, hdwp: isize) -> isize {
        let mut flags = SWP_NOZORDER | SWP_NOACTIVATE;
        if !self.size_dirty {
            flags |= SWP_NOSIZE;
        }
        if !self.pos_dirty {
            flags |= SWP_NOMOVE;
        }
        if self.vis_dirty {
            flags |= if self.visible {
                SWP_SHOWWINDOW
            } else {
                SWP_HIDEWINDOW
            };
        }
        // NOTE: no SWP_NOREDRAW. A window that is resized (or shown) must
        // repaint, otherwise only a stale sliver of its surface shows.
        // (Pure moves would be fine with NOREDRAW, but resizes are the common
        // case here since boxes change size every frame.)

        self.pos_dirty = false;
        self.size_dirty = false;
        self.vis_dirty = false;

        unsafe { DeferWindowPos(hdwp, self.hwnd, None, self.x, self.y, self.w, self.h, flags) }
    }
}

/// All tiles, drawn together with a single batched DeferWindowPos pass.
struct TileGrid {
    tiles: Vec<TileWindow>,
}

impl TileGrid {
    fn new(hinst: HINSTANCE) -> Self {
        let now = Instant::now();
        let tiles = (0..MAX_WINDOWS).map(|_| TileWindow::create(hinst)).collect();
        println!("Created {} tiles in {:?}", MAX_WINDOWS, now.elapsed());
        Self { tiles }
    }

    fn draw(&mut self) {
        let changed = self.tiles.iter().filter(|t| t.dirty()).count();
        if changed == 0 {
            return;
        }

        let mut hdwp = unsafe { BeginDeferWindowPos(changed as i32) };
        assert!(hdwp != 0);

        for tile in self.tiles.iter_mut().filter(|t| t.dirty()) {
            hdwp = tile.apply(hdwp);
            assert!(hdwp != 0);
        }

        unsafe { EndDeferWindowPos(hdwp) };
    }
}

/// One box from boxes.bin. repr(packed) + NonZeroU8 fields mean
/// `Option<WinCoords>` is also 4 bytes, with None == the (0,0,0,0)
/// frame separator -- so the whole file is just a flat slice, no parsing.
#[repr(packed)]
#[derive(Debug, Copy, Clone)]
struct WinCoords {
    x: u8,
    y: u8,
    w: NonZeroU8,
    h: NonZeroU8,
}

fn main() {
    // The video timeline starts at process launch. This must be set BEFORE any
    // audio initialization: on this machine the (virtual) audio device can
    // take seconds to initialize, and the video must not wait for it.
    let playback_start = Instant::now();

    // Best-effort attach to the parent console so println output is visible
    // when launched from a terminal.
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }

    let hinst = register_classes().expect("failed to register window classes");

    // Physical pixels (PerMonitorV2 manifest). Full screen, taskbar included.
    let scr_w = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let scr_h = unsafe { GetSystemMetrics(SM_CYSCREEN) };

    // Integer cell size, 4:3 letterboxed and centred.
    let cell = (scr_w / BASE_WIDTH as i32).min(scr_h / BASE_HEIGHT as i32);
    let content_w = cell * BASE_WIDTH as i32;
    let content_h = cell * BASE_HEIGHT as i32;
    let offset_x = (scr_w - content_w) / 2;
    let offset_y = (scr_h - content_h) / 2;
    println!(
        "screen {}x{} cell {} content {}x{} offset ({},{})",
        scr_w, scr_h, cell, content_w, content_h, offset_x, offset_y
    );

    // Full-screen black backdrop, created first so it sits at the bottom of
    // the topmost stack. The "black" parts of the video (hidden tiles) now
    // show as black regardless of the desktop wallpaper.
    let bg = unsafe {
        CreateWindowExA(
            WS_EX_TOPMOST,
            PCSTR(BG_CLASS.as_ptr() as _),
            s!(""),
            WS_POPUP | WS_VISIBLE,
            0,
            0,
            scr_w,
            scr_h,
            None,
            None,
            hinst,
            None,
        )
    };
    assert!(bg.0 != 0);

    // Escape quits (works even though borderless tiles never take focus).
    unsafe {
        RegisterHotKey(Some(bg), HOTKEY_ID, HOT_KEY_MODIFIERS(0), VK_ESCAPE.0 as u32)
            .expect("failed to register Escape hotkey");
    }

    let frames_raw = include_bytes_zstd!("assets/boxes.bin", 22);

    // Parse the box stream explicitly. NOTE: we must NOT reinterpret the raw
    // bytes as `&[Option<WinCoords>]` -- the compiler is free to reorder
    // (packed) struct fields for the niche optimization, so the in-memory
    // layout of Option<WinCoords> is NOT [x,y,w,h] on modern rustc. Explicit
    // parsing is always correct. (0,0,0,0) separates frames.
    let mut parsed: Vec<Option<WinCoords>> = Vec::with_capacity(frames_raw.len() / 4);
    for chunk in frames_raw.chunks_exact(4) {
        let (x, y, w, h) = (chunk[0], chunk[1], chunk[2], chunk[3]);
        if (x, y, w, h) == (0, 0, 0, 0) {
            parsed.push(None);
        } else {
            parsed.push(Some(WinCoords {
                x,
                y,
                w: NonZeroU8::new(w).expect("box width must be non-zero"),
                h: NonZeroU8::new(h).expect("box height must be non-zero"),
            }));
        }
    }
    let frames = parsed;
    let mut frames_iter = frames.iter();

    let mut grid = TileGrid::new(hinst);

    // Audio: play best-effort. The video timeline uses a wall clock instead of
    // the audio clock -- the audio backend's clock is unreliable in some
    // environments (virtual/remote audio devices), but video must stay smooth.
    // If the audio device fails to initialize, the video still plays silently.
    let cursor = Cursor::new(include_bytes!("../assets/bad apple.ogg"));
    // Kept alive for the whole run (drop guard keeps the audio backend going);
    // the underscore name suppresses "never used" warnings.
    let mut _audio_manager: Option<AudioManager<DefaultBackend>> = None;
    match AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()) {
        Ok(mut manager) => {
            match StreamingSoundData::from_cursor(cursor, StreamingSoundSettings::new()) {
                Ok(sound_data) => {
                    if manager.play(sound_data).is_err() {
                        println!("audio: failed to start playback (video only)");
                    }
                }
                Err(_) => println!("audio: failed to decode ogg (video only)"),
            }
            _audio_manager = Some(manager); // keep the audio backend alive
        }
        Err(_) => println!("audio: no audio backend available (video only)"),
    }

    let mut next_tick: u64 = 0; // index of the next frame to display

    unsafe { SetTimer(None, 1, 16, None) }; // poll ~62Hz, frames advance at 30Hz

    'outer: loop {
        let mut msg: MSG = unsafe { std::mem::zeroed() };

        while unsafe { PeekMessageA(&mut msg, None, 0, 0, PM_REMOVE).as_bool() } {
            if msg.message == WM_QUIT {
                break 'outer;
            } else if msg.message == WM_TIMER {
                // Frame index the wall clock says we should be at (30fps).
                let target = (playback_start.elapsed().as_secs_f64() * 30.0) as u64;
                // Not time for the next frame yet.
                if next_tick > target {
                    continue;
                }
                // Wall clock got ahead (message loop stalled): skip missed frames.
                while next_tick < target {
                    loop {
                        let Some(val) = frames_iter.next() else {
                            break 'outer;
                        };
                        if val.is_none() {
                            break;
                        }
                    }
                    next_tick += 1;
                }

                // Place one tile per box in the current frame.
                let mut tiles = grid.tiles.iter_mut();
                loop {
                    let Some(val) = frames_iter.next() else {
                        break 'outer;
                    };
                    let Some(c) = val else {
                        break;
                    };
                    let tile = tiles.next().unwrap();
                    tile.set_rect(
                        offset_x + c.x as i32 * cell,
                        offset_y + c.y as i32 * cell,
                        c.w.get() as i32 * cell,
                        c.h.get() as i32 * cell,
                    );
                    tile.set_visible(true);
                }
                // Everything left over is black in this frame.
                for tile in tiles {
                    tile.set_visible(false);
                }

                grid.draw();
                next_tick += 1;
            }

            unsafe {
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }
        }
    }

    unsafe {
        let _ = UnregisterHotKey(Some(bg), HOTKEY_ID);
    }
    println!("done");
}
