//! a small library of utility functions for interacting with the
//! wezterm command line interface
use std::{env, usize};

/// acquires and returns the current directory as a `String``
fn current_dir() -> String {
    env::current_dir()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string()
}

/// handle to wez term cli
pub struct WTCli {}

impl WTCli {
    pub fn new() -> Self {
        WTCli {}
    }

    /// Spans a new pane, setting CWD to the current directory
    /// (otherwise it would be set to $HOME).
    /// The passed in command is wrapped in a zsh invocation.
    /// This is necessary, as otherwise, the environment and
    /// more specifically the $PATH variable would lack the
    /// user's settings as defined in her ~/.zshrc or equivalent.
    pub fn spawn(&self, cmd: &str) -> WTPane {
        let pane_id = wt_cli_exec(
            [
                vec!["spawn"],
                current_dir_vec(&current_dir()),
                wrap_in_shell(&shell(), cmd),
            ]
            .concat(),
        );
        WTPane { id: pane_id }
    }

    pub fn active_pane(&self) -> WTPane {
        let pane_id = env::var("WEZTERM_PANE").unwrap();
        WTPane { id: pane_id }
    }
}

impl Default for WTCli {
    fn default() -> Self {
        Self::new()
    }
}

/// executes the given `cmd` as a sub process and
/// returns its output as a string
fn wt_cli_exec(cmd: Vec<&str>) -> String {
    let cmd = [vec!["wezterm", "cli"], cmd].concat();
    crate::exec(cmd)
}

/// returns a vector with the `--cwd`` option as first element
/// and the passed in `dir` string as second
fn current_dir_vec(dir: &str) -> Vec<&str> {
    vec!["--cwd", dir]
}

/// returns value of SHELL variable
fn shell() -> String {
    match env::var("SHELL") {
        Ok(val) => val,
        Err(_) => "zsh".to_string(),
    }
}

/// creates vector of shell command and -c option
fn wrap_in_shell<'a>(shell_cmd: &'a str, cmd: &'a str) -> Vec<&'a str> {
    vec![shell_cmd, "-c", cmd]
}

/// returns a vector with `--pane-id` and the
/// pane id as members
fn pane_id_vec(id: &str) -> Vec<&str> {
    vec!["--pane-id", id]
}

/// encapsulates a wezterm pane id.
/// has method to split pane
pub struct WTPane {
    /// the wezterm pane id
    pub id: String,
}

/// SplitOpts try to capture wezterm cli split options in a type-safe manner
pub struct SplitOptsBuilder {
    id: String,
    cmd: String,
    opts: Vec<String>,
}

impl SplitOptsBuilder {
    /// add a percentage option
    pub fn percent(&mut self, percentage: usize) -> &mut SplitOptsBuilder {
        self.opts.push("--percent".to_string());
        self.opts.push(format!("{}", percentage));
        self
    }
    /// add a bottom option
    pub fn bottom(&mut self) -> &mut SplitOptsBuilder {
        self.opts.push("--bottom".to_string());
        self
    }
    /// add a right option
    pub fn right(&mut self) -> &mut SplitOptsBuilder {
        self.opts.push("--right".to_string());
        self
    }
    /// add a top-level option
    pub fn top_level(&mut self) -> &mut SplitOptsBuilder {
        self.opts.push("--top-level".to_string());
        self
    }
    /// transform to a vev<&str> needed by exec
    pub fn as_vec(&self) -> Vec<&str> {
        self.opts.iter().map(AsRef::as_ref).collect()
    }

    pub fn exec(&self) -> WTPane {
        let pane_id = wt_cli_exec(
            [
                vec!["split-pane"],
                pane_id_vec(&self.id),
                current_dir_vec(&current_dir()),
                self.as_vec(),
                wrap_in_shell(&shell(), &self.cmd),
            ]
            .concat(),
        );
        WTPane { id: pane_id }
    }
}
impl WTPane {
    /// splits the current pane and launches `cmd`.
    /// the passed in vector of `opts` allows for customization:
    /// how big the new split is supposed to be and where should
    /// it be located.
    pub fn split(&self, cmd: &str) -> SplitOptsBuilder {
        SplitOptsBuilder {
            id: self.id.clone(),
            cmd: cmd.to_owned(),
            opts: Vec::new(),
        }
    }
    /// activates the pane identified by `self`, which means, it gets the focus
    pub fn activate(&self) {
        wt_cli_exec([vec!["activate-pane"], pane_id_vec(&self.id)].concat());
    }

    /// kill a pane
    pub fn kill(&self) {
        wt_cli_exec([vec!["kill-pane"], pane_id_vec(&self.id)].concat());
    }
}
