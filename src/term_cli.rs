use std::env;

use crate::alacritty_cli::AlaCli;
use crate::wezterm_cli::{WTCli, WTPane};

const DEFAULT_EDITOR: &str = "nvim";

/// adapter to terminal remote control, or command-line interface.
/// delegates calls to concrete cli interface implementations.
pub enum TermCli {
    WezTerm,
    Alacritty,
    Kitty,
    Other,
}
fn get_editor() -> String {
    // we look-up the users preferred editor via the environment
    // variable.
    match env::var("EDITOR") {
        Ok(val) => val,
        Err(_) => DEFAULT_EDITOR.to_string(),
    }
}

impl TermCli {
    pub fn get() -> Self {
        if env::var("WEZTERM_PANE").is_ok() {
            TermCli::WezTerm
        } else if env::var("KITTY_WINDOW_ID").is_ok() {
            panic!("kitty not supported.");
        } else if env::var("ALACRITTY_WINDOW_ID").is_ok() {
            TermCli::Alacritty
        } else {
            panic!("unknown terminal not supported.");
        }
    }

    pub fn get_active_windows_handle(&self) -> usize {
        match self {
            Self::WezTerm => WTCli::new().active_pane().id.parse::<usize>().unwrap(),
            // alacritty has no remote api — the editor runs in the current terminal
            Self::Alacritty => 0,
            _ => 0,
        }
    }
    /// opens an editor and blocks on the call
    pub fn open_editor(&self, source_file: &str) {
        match self {
            Self::WezTerm | Self::Alacritty => {
                crate::spawn(vec![&get_editor(), source_file]);
            }
            _ => {}
        }
    }

    /// opens an editor in a new window asynchronously an returns a numeric handle to it
    pub fn spawn_editor(&self, source_file: &str) -> usize {
        match self {
            Self::WezTerm => {
                let wt_cli = WTCli::new();
                let editor_pane = wt_cli.spawn(&format!("{} {}", get_editor(), source_file));
                editor_pane.id.parse::<usize>().unwrap()
            }
            Self::Alacritty => {
                let window = AlaCli::new()
                    .spawn_window(&format!("{} {}", get_editor(), source_file));
                window.pid as usize
            }
            _ => 0,
        }
    }

    pub fn exec_matopdf(&self, source_file: &str, lang: &str, t_handle: usize) -> usize {
        match self {
            Self::WezTerm => {
                let editor_pane = WTPane {
                    id: t_handle.to_string(),
                };
                let mato_pane = editor_pane
                    .split(format!("matopdf -w -V -l {lang} {source_file}").as_str())
                    .percent(10)
                    .bottom()
                    .exec();
                mato_pane.id.parse::<usize>().unwrap()
            }
            Self::Alacritty => {
                let window = AlaCli::new()
                    .spawn_window(&format!("matopdf -w -V -l {lang} {source_file}"));
                window.pid as usize
            }
            _ => 0,
        }
    }

    pub fn exec_termpdf(&self, target_file: &str, t_handle: usize) -> usize {
        match self {
            Self::WezTerm => {
                let editor_pane = WTPane {
                    id: t_handle.to_string(),
                };
                let termpdf_pane = editor_pane
                    .split(&format!("termpdf.py {target_file}"))
                    .right()
                    .exec();
                termpdf_pane.id.parse::<usize>().unwrap()
            }
            Self::Alacritty => {
                std::process::Command::new("xdg-open")
                    .arg(target_file)
                    .spawn()
                    .expect("failed to open pdf");
                0
            }
            _ => 0,
        }
    }

    /// sets the focus to the given pane or window
    /// identified by the given handle
    pub fn focus(&self, t_handle: usize) {
        match self {
            Self::WezTerm => {
                let pane = WTPane {
                    id: t_handle.to_string(),
                };
                pane.activate();
            }
            // no-op for alacritty: focus management is left to the window manager
            Self::Alacritty => {}
            _ => {}
        }
    }

    /// closes, or kills the pane or window identified
    /// by the given handle
    pub fn close(&self, t_handle: usize) {
        match self {
            TermCli::WezTerm => {
                let pane = WTPane {
                    id: t_handle.to_string(),
                };
                pane.kill();
            }
            TermCli::Alacritty => {
                if t_handle != 0 {
                    crate::alacritty_cli::AlaWindow { pid: t_handle as u32 }.kill();
                }
            }
            TermCli::Kitty => todo!(),
            TermCli::Other => todo!(),
        }
    }
}
