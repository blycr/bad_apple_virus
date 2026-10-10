//! One executable keeps both visual styles without duplicating the player.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Borderless,
    Windows,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Options {
    pub mode: Mode,
    pub muted: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Play(Options),
    Help,
    Version,
}

pub const HELP: &str = "Bad Apple Windows Player\n\n\
Usage: bad_apple.exe [--mode borderless|windows] [--mute]\n\n\
  --mode borderless  White tiles on a black backdrop (default)\n\
  --mode windows     Captioned windows; small tiles stay borderless\n\
  --mute             Play silently\n\
  --help, -h         Show this help\n\
  --version, -V      Show the version\n\n\
Ctrl+Alt+F8: pause/resume picture and sound\n\
Ctrl+Alt+F9: quit\n";

pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut options = Options::default();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--mode" => {
                options.mode = match args.next().as_deref() {
                    Some("borderless") => Mode::Borderless,
                    Some("windows") => Mode::Windows,
                    _ => return Err("--mode expects borderless or windows".into()),
                };
            }
            "--mute" => options.muted = true,
            "--help" | "-h" => return Ok(Command::Help),
            "--version" | "-V" => return Ok(Command::Version),
            _ => return Err(format!("Unknown option: {arg}\n\n{HELP}")),
        }
    }
    Ok(Command::Play(options))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn default_and_legacy_modes_share_the_same_command() {
        assert_eq!(parse(args(&[])), Ok(Command::Play(Options::default())));
        assert_eq!(
            parse(args(&["--mode", "windows", "--mute"])),
            Ok(Command::Play(Options {
                mode: Mode::Windows,
                muted: true
            }))
        );
        assert_eq!(
            parse(args(&["--mode", "borderless"])),
            Ok(Command::Play(Options::default()))
        );
    }

    #[test]
    fn invalid_arguments_cannot_start_playback() {
        assert!(parse(args(&["--mode"])).is_err());
        assert!(parse(args(&["--mode", "high-resolution"])).is_err());
        assert!(parse(args(&["--unknown"])).is_err());
    }

    #[test]
    fn information_commands_do_not_start_playback() {
        assert_eq!(parse(args(&["--help"])), Ok(Command::Help));
        assert_eq!(parse(args(&["--version"])), Ok(Command::Version));
    }
}
