//! Shared, bounded Win32 window renderer for both visual styles.
use crate::options::Mode;
use bad_apple::video::{Rect, Viewport};
use windows::{
    core::*,
    Win32::{
        Foundation::*, Graphics::Gdi::*, System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::*,
    },
};

const TILE_CLASS: PCSTR = s!("BadAppleTile");
const BG_CLASS: PCSTR = s!("BadAppleBG");

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    if msg == WM_CLOSE {
        PostQuitMessage(0);
        return LRESULT(0);
    }
    DefWindowProcA(hwnd, msg, w, l)
}

struct Tile {
    hwnd: HWND,
    rect: (i32, i32, i32, i32),
    visible: bool,
    decorated: bool,
    position_dirty: bool,
    size_dirty: bool,
    visibility_dirty: bool,
    style_dirty: bool,
}

impl Tile {
    fn set_rect(&mut self, rect: (i32, i32, i32, i32), mode: Mode) {
        self.position_dirty = (self.rect.0, self.rect.1) != (rect.0, rect.1);
        self.size_dirty = (self.rect.2, self.rect.3) != (rect.2, rect.3);
        self.rect = rect;
        // Smaller captioned windows are forcibly enlarged by Windows. Keep
        // contour fragments borderless so their outer edges remain exact.
        let decorated = mode == Mode::Windows && rect.2 >= 120 && rect.3 >= 64;
        self.style_dirty = self.decorated != decorated;
        self.decorated = decorated;
    }

    fn set_visible(&mut self, visible: bool) {
        self.visibility_dirty = self.visible != visible;
        self.visible = visible;
    }

    fn dirty(&self) -> bool {
        self.position_dirty || self.size_dirty || self.visibility_dirty || self.style_dirty
    }

    fn needs_repaint(&self) -> bool {
        self.visible && (self.size_dirty || self.visibility_dirty || self.style_dirty)
    }

    fn apply(&mut self, batch: isize) -> Result<isize> {
        let mut flags = SWP_NOZORDER | SWP_NOACTIVATE;
        if self.style_dirty {
            let style = if self.decorated {
                WS_POPUP | WS_CAPTION | WS_SYSMENU
            } else {
                WS_POPUP
            };
            unsafe {
                SetWindowLongPtrA(self.hwnd, GWL_STYLE, style.0 as isize);
            }
            flags |= SWP_FRAMECHANGED;
        }
        if !self.position_dirty {
            flags |= SWP_NOMOVE;
        }
        if !self.size_dirty {
            flags |= SWP_NOSIZE;
        }
        if self.visibility_dirty {
            flags |= if self.visible {
                SWP_SHOWWINDOW
            } else {
                SWP_HIDEWINDOW
            };
        }
        if !self.size_dirty && !self.visibility_dirty && !self.style_dirty {
            flags |= SWP_NOREDRAW;
        }
        let (x, y, width, height) = self.rect;
        let batch = unsafe { DeferWindowPos(batch, self.hwnd, None, x, y, width, height, flags) };
        if batch == 0 {
            return Err(Error::from_win32());
        }
        self.position_dirty = false;
        self.size_dirty = false;
        self.visibility_dirty = false;
        self.style_dirty = false;
        Ok(batch)
    }
}

pub struct Renderer {
    mode: Mode,
    instance: HINSTANCE,
    viewport: Viewport,
    tiles: Vec<Tile>,
    backdrop: HWND,
    white: HBRUSH,
    black: HBRUSH,
    tile_class_registered: bool,
    bg_class_registered: bool,
}

impl Renderer {
    pub fn new(mode: Mode, count: usize, grid_width: i32, grid_height: i32) -> Result<Self> {
        let instance = unsafe { GetModuleHandleW(PCWSTR::null()) }?;
        let width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        let height = unsafe { GetSystemMetrics(SM_CYSCREEN) };
        // Construct the resource owner first: partial startup failures clean up
        // every window, registered class, and brush already created.
        let mut renderer = Self {
            mode,
            instance,
            viewport: Viewport::fit(width, height, grid_width, grid_height),
            tiles: Vec::with_capacity(count),
            backdrop: HWND(0),
            white: HBRUSH(0),
            black: HBRUSH(0),
            tile_class_registered: false,
            bg_class_registered: false,
        };
        unsafe {
            renderer.white = CreateSolidBrush(COLORREF(0xFFFFFF));
            renderer.black = CreateSolidBrush(COLORREF(0));
            if renderer.white.0 == 0 || renderer.black.0 == 0 {
                return Err(Error::from_win32());
            }
            for (name, brush) in [(TILE_CLASS, renderer.white), (BG_CLASS, renderer.black)] {
                let class = WNDCLASSA {
                    lpfnWndProc: Some(wnd_proc),
                    hInstance: instance,
                    lpszClassName: name,
                    hbrBackground: brush,
                    ..Default::default()
                };
                if RegisterClassA(&class) == 0 {
                    return Err(Error::from_win32());
                }
                if name == TILE_CLASS {
                    renderer.tile_class_registered = true;
                } else {
                    renderer.bg_class_registered = true;
                }
            }
            renderer.backdrop = CreateWindowExA(
                WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                BG_CLASS,
                s!("Bad Apple"),
                WS_POPUP | WS_VISIBLE,
                0,
                0,
                width,
                height,
                None,
                None,
                instance,
                None,
            );
            if renderer.backdrop.0 == 0 {
                return Err(Error::from_win32());
            }
            for _ in 0..count {
                let hwnd = CreateWindowExA(
                    WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                    TILE_CLASS,
                    s!("Bad Apple!!"),
                    WS_POPUP,
                    0,
                    0,
                    1,
                    1,
                    None,
                    None,
                    instance,
                    None,
                );
                if hwnd.0 == 0 {
                    return Err(Error::from_win32());
                }
                renderer.tiles.push(Tile {
                    hwnd,
                    rect: (0, 0, 1, 1),
                    visible: false,
                    decorated: false,
                    position_dirty: false,
                    size_dirty: false,
                    visibility_dirty: false,
                    style_dirty: false,
                });
            }
        }
        Ok(renderer)
    }

    pub fn draw(&mut self, frame: &[Rect]) -> Result<()> {
        let mut tiles = self.tiles.iter_mut();
        for rect in frame {
            let mapped = self.viewport.map(rect);
            if mapped.2 == 0 || mapped.3 == 0 {
                continue;
            }
            let tile = tiles
                .next()
                .expect("video exceeded its validated window budget");
            tile.set_rect(mapped, self.mode);
            tile.set_visible(true);
        }
        for tile in tiles {
            tile.set_visible(false);
        }
        let changed = self.tiles.iter().filter(|t| t.dirty()).count();
        if changed == 0 {
            return Ok(());
        }
        let repaint: Vec<_> = self
            .tiles
            .iter()
            .filter(|t| t.needs_repaint())
            .map(|t| t.hwnd)
            .collect();
        let mut batch = unsafe { BeginDeferWindowPos(changed as i32) };
        if batch == 0 {
            return Err(Error::from_win32());
        }
        for tile in self.tiles.iter_mut().filter(|t| t.dirty()) {
            batch = tile.apply(batch)?;
        }
        if !unsafe { EndDeferWindowPos(batch) }.as_bool() {
            return Err(Error::from_win32());
        }
        // Paint new surface sizes before DWM presents them, avoiding black holes.
        for hwnd in repaint {
            if !unsafe {
                RedrawWindow(hwnd, None, None, RDW_INVALIDATE | RDW_ERASE | RDW_UPDATENOW)
            }
            .as_bool()
            {
                return Err(Error::from_win32());
            }
        }
        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            for tile in &self.tiles {
                DestroyWindow(tile.hwnd);
            }
            if self.backdrop.0 != 0 {
                DestroyWindow(self.backdrop);
            }
            if self.tile_class_registered {
                UnregisterClassA(TILE_CLASS, self.instance);
            }
            if self.bg_class_registered {
                UnregisterClassA(BG_CLASS, self.instance);
            }
            if self.white.0 != 0 {
                DeleteObject(HGDIOBJ(self.white.0));
            }
            if self.black.0 != 0 {
                DeleteObject(HGDIOBJ(self.black.0));
            }
        }
    }
}
