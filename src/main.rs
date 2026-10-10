#![windows_subsystem = "windows"]

mod controls;
mod options;
mod renderer;

use bad_apple::playback::Playback;
use bad_apple::video::{Video, FPS};
use controls::{Hotkeys, PAUSE_ID, QUIT_ID};
use include_bytes_zstd::include_bytes_zstd;
use kira::sound::streaming::StreamingSoundSettings;
use options::{Command, Options};
use renderer::Renderer;
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        System::Console::{AttachConsole, ATTACH_PARENT_PROCESS},
        UI::WindowsAndMessaging::*,
    },
};

struct Timer(usize);
impl Timer {
    fn start() -> Result<Self> {
        let id = unsafe { SetTimer(None, 1, 16, None) };
        if id == 0 {
            return Err(Error::from_win32());
        }
        Ok(Self(id))
    }
}
impl Drop for Timer {
    fn drop(&mut self) {
        unsafe {
            KillTimer(None, self.0);
        }
    }
}

fn play(options: Options) -> Result<()> {
    // Register before covering the desktop so a conflict has no visual side effects.
    let _hotkeys = Hotkeys::register().map_err(|error| {
        Error::new(
            error.code(),
            "无法注册 Ctrl+Alt+F8/F9。请关闭其他正在运行的播放器，或释放已占用的快捷键。".into(),
        )
    })?;
    let raw = include_bytes_zstd!("assets/boxes.bin", 22);
    let video = Video::from_legacy(&raw).map_err(|error| Error::new(E_FAIL, error.into()))?;
    let mut renderer = Renderer::new(options.mode, video.max_windows(), video.width, video.height)?;
    println!(
        "Mode: {:?}; {}x{}; at most {} tile windows",
        options.mode,
        video.width,
        video.height,
        video.max_windows()
    );

    // Windows and physical sizing are ready before the asynchronous audio start.
    let settings = StreamingSoundSettings::new().volume(if options.muted { 0.0 } else { 1.0 });
    let mut playback = Playback::start(include_bytes!("../assets/bad apple.ogg"), settings);
    let _timer = Timer::start()?;
    let mut shown_frame = None;

    // GetMessage blocks until input/timer events, including while paused.
    loop {
        let mut msg = MSG::default();
        let result = unsafe { GetMessageA(&mut msg, None, 0, 0) };
        if result.0 == -1 {
            return Err(Error::from_win32());
        }
        if result.0 == 0 {
            break;
        }
        match msg.message {
            WM_HOTKEY => match msg.wParam.0 as i32 {
                QUIT_ID => break,
                PAUSE_ID => {
                    playback.toggle_pause().map_err(|error| {
                        Error::new(E_FAIL, format!("Pause/resume failed: {error:?}").into())
                    })?;
                }
                _ => {}
            },
            WM_TIMER if msg.wParam.0 == _timer.0 && !playback.is_paused() => {
                let target = (playback.position() * FPS) as usize;
                if shown_frame == Some(target) {
                    continue;
                }
                let Some(frame) = video.frame(target) else {
                    break;
                };
                renderer.draw(frame)?;
                shown_frame = Some(target);
            }
            _ => unsafe {
                TranslateMessage(&msg);
                DispatchMessageA(&msg);
            },
        }
    }
    // Timer, audio, windows/brushes/classes, and hotkeys drop on every exit path.
    Ok(())
}

fn show_message(message: &str, error: bool) {
    println!("{message}");
    let text = HSTRING::from(message);
    unsafe {
        MessageBoxW(
            None,
            PCWSTR(text.as_ptr()),
            w!("Bad Apple"),
            MB_OK
                | if error {
                    MB_ICONERROR
                } else {
                    MB_ICONINFORMATION
                },
        );
    }
}

fn main() {
    let console_attached = unsafe { AttachConsole(ATTACH_PARENT_PROCESS) }.as_bool();
    let command = match options::parse(std::env::args().skip(1)) {
        Ok(command) => command,
        Err(error) => {
            show_message(&error, true);
            std::process::exit(1);
        }
    };
    match command {
        Command::Help | Command::Version => {
            let message = if command == Command::Help {
                options::HELP.to_owned()
            } else {
                format!("Bad Apple Windows Player {}", env!("CARGO_PKG_VERSION"))
            };
            if console_attached {
                println!("{message}");
            } else {
                show_message(&message, false);
            }
        }
        Command::Play(options) => {
            if let Err(error) = play(options) {
                // play() has already cleaned up the screen and global shortcuts.
                show_message(&error.to_string(), true);
                std::process::exit(1);
            }
        }
    }
}
