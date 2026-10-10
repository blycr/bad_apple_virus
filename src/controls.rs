//! Uncommon global combinations work even when the popup tiles lack focus.
use windows::{core::*, Win32::UI::Input::KeyboardAndMouse::*};

pub const PAUSE_ID: i32 = 1;
pub const QUIT_ID: i32 = 2;

pub struct Hotkeys {
    registered: Vec<i32>,
}

impl Hotkeys {
    pub fn register() -> Result<Self> {
        let mut keys = Self {
            registered: Vec::new(),
        };
        for (id, key) in [(PAUSE_ID, VK_F8), (QUIT_ID, VK_F9)] {
            if !unsafe {
                RegisterHotKey(None, id, MOD_CONTROL | MOD_ALT | MOD_NOREPEAT, key.0 as u32)
            }
            .as_bool()
            {
                return Err(Error::from_win32());
            }
            keys.registered.push(id);
        }
        Ok(keys)
    }
}

impl Drop for Hotkeys {
    fn drop(&mut self) {
        for id in &self.registered {
            unsafe {
                UnregisterHotKey(None, *id);
            }
        }
    }
}
