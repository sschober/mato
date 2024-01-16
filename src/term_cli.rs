use std::env;

use crate::wezterm_cli::{WTCli, WTPane};

const DEFAULT_EDITOR: &str = "nvim";

/// adapter to terminal remote control, or command-line interface
pub enum TermCli {
    WezTerm,
    Kitty,
    Other,
}
impl TermCli {
    pub fn get() -> Self {
        if env::var("WEZTERM_PANE").is_ok() {
            TermCli::WezTerm
        } else if env::var("KITTY_WINDOW_ID").is_ok() {
            panic!("kitty not supported.");
        } else {
            panic!("unknown terminal not supported.");
        }
    }

    /// opens an editor an returns a numeric handle to it
    pub fn open_editor(&self, source_file: &str) -> usize {
        // we look up the users preferred editor via the environment
        // variable.
        let editor_cmd = match env::var("EDITOR") {
            Ok(val) => val,
            Err(_) => DEFAULT_EDITOR.to_string(),
        };

        match self {
            Self::WezTerm => {
                let wt_cli = WTCli::new();
                // SPAWN the EDITOR pane!
                let editor_pane = wt_cli.spawn(&format!("{} {}", editor_cmd, source_file));
                editor_pane.id.parse::<usize>().unwrap()
            }
            Self::Kitty => 0,
            Self::Other => 0,
        }
    }

    pub fn exec_matopdf(&self, source_file: &str, t_handle: usize) -> usize {
        match self {
            Self::WezTerm => {
                let editor_pane = WTPane {
                    id: t_handle.to_string(),
                };
                let mato_pane = editor_pane
                    .split(format!("matopdf -w -v {}", source_file).as_str())
                    .percent(10)
                    .bottom()
                    .exec();
                mato_pane.id.parse::<usize>().unwrap()
            }
            Self::Kitty => 0,
            Self::Other => 0,
        }
    }

    pub fn exec_termpdf(&self, target_file: &str, t_handle: usize) -> usize {
        match self {
            Self::WezTerm => {
                let editor_pane = WTPane {
                    id: t_handle.to_string(),
                };
                let termpdf_pane = editor_pane
                    .split(&format!("termpdf.py {}", target_file))
                    .top_level()
                    .right()
                    .exec();
                termpdf_pane.id.parse::<usize>().unwrap()
            }
            Self::Kitty => 0,
            Self::Other => 0,
        }
    }

    pub fn focus_editor(&self, t_handle: usize) {
        match self {
            Self::WezTerm => {
                let editor_pane = WTPane {
                    id: t_handle.to_string(),
                };
                editor_pane.activate();
            }
            Self::Kitty => return,
            Self::Other => return,
        }
    }
}
