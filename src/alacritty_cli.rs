//! a small library of utility functions for spawning new alacritty windows.
//! since alacritty has no remote control api, handles are process ids (PIDs).
use std::env;
use std::process::Command;

fn current_dir() -> String {
    env::current_dir()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string()
}

fn shell() -> String {
    match env::var("SHELL") {
        Ok(val) => val,
        Err(_) => "zsh".to_string(),
    }
}

pub struct AlaCli {}

impl AlaCli {
    pub fn new() -> Self {
        AlaCli {}
    }

    /// Spawns a new alacritty window running `cmd` wrapped in the user's shell.
    /// Returns an `AlaWindow` whose `pid` identifies the alacritty process.
    pub fn spawn_window(&self, cmd: &str) -> AlaWindow {
        let child = Command::new("alacritty")
            .args(["--working-directory", &current_dir()])
            .args(["-e", &shell(), "-c", cmd])
            .spawn()
            .expect("failed to spawn alacritty window");
        AlaWindow { pid: child.id() }
    }
}

impl Default for AlaCli {
    fn default() -> Self {
        Self::new()
    }
}

/// handle to an alacritty window, identified by the PID of the alacritty process
pub struct AlaWindow {
    pub pid: u32,
}

impl AlaWindow {
    pub fn kill(&self) {
        Command::new("kill")
            .args(["-TERM", &self.pid.to_string()])
            .status()
            .ok();
    }
}
