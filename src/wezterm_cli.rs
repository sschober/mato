//! a small library of utility functions for interacting with the
//! wezterm command line interface
use std::{env, process::Command, usize};

/// acquires and returns the current directory as a `String``
fn current_dir() -> String {
    env::current_dir()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string()
}

/// executes the given `cmd` as a sub process and
/// returns its output as a string
fn exec(cmd: Vec<&str>) -> String {
    eprintln!("exec: {:?}", cmd);
    let out = Command::new("/usr/bin/env")
        .args(cmd)
        .output()
        .expect("error executing spawn command")
        .stdout;
    if !out.is_empty() {
        String::from_utf8(out)
            .unwrap()
            .strip_suffix('\n')
            .unwrap()
            .to_string()
    } else {
        String::new()
    }
}

/// returns a vector with `wezterm` and `cli` as members
fn wezterm_cli_vec() -> Vec<&'static str> {
    vec!["wezterm", "cli"]
}

/// returns a vector with `zsh`, `-c` and `cmd` as constituents
/// effectively wrapping a command in a shell
fn zsh_c_vec(cmd: &str) -> Vec<&str> {
    vec!["zsh", "-c", cmd]
}

/// returns a vector with the `--cwd`` option as first element
/// and the passed in `dir` string as second
fn current_dir_vec(dir: &str) -> Vec<&str> {
    vec!["--cwd", dir]
}

/// Spans a new pane, setting CWD to the current directory
/// (otherwise it would be set to $HOME).
/// The passed in command is wrapped in a zsh invocation.
/// This is necessary, as otherwise, the environment and
/// more specifically the $PATH variable would lack the
/// user's settings as defined in her ~/.zshrc or equivalent.
pub fn spawn(cmd: &str) -> Pane {
    let pane_id = exec(
        [
            wezterm_cli_vec(),
            vec!["spawn"],
            current_dir_vec(&current_dir()),
            zsh_c_vec(cmd),
        ]
        .concat(),
    );
    Pane { id: pane_id }
}

/// encapsulates a wezterm pane id.
/// has method to split pane
pub struct Pane {
    /// the wezterm pane id
    pub id: String,
}

pub struct SplitOpts {
    opts: Vec<String>,
}

impl SplitOpts {
    pub fn new() -> SplitOpts {
        SplitOpts { opts: Vec::new() }
    }
    pub fn percent(&mut self, percentage: usize) -> &mut SplitOpts {
        self.opts.push("--percent".to_string());
        self.opts.push(format!("{}", percentage));
        self
    }
    pub fn bottom(&mut self) -> &mut SplitOpts {
        self.opts.push("--bottom".to_string());
        self
    }
    pub fn right(&mut self) -> &mut SplitOpts {
        self.opts.push("--right".to_string());
        self
    }
    pub fn top_level(&mut self) -> &mut SplitOpts {
        self.opts.push("--top-level".to_string());
        self
    }
    pub fn as_vec(&self) -> Vec<&str> {
        self.opts.iter().map(AsRef::as_ref).collect()
    }
}
impl Pane {
    /// returns a vector with `--pane-id` and the
    /// pane id as members
    fn pane_id_vec(&self) -> Vec<&str> {
        vec!["--pane-id", self.id.as_str()]
    }
    /// splits the current pane and launches `cmd`.
    /// the passed in vector of `opts` allows for customization:
    /// how big the new split is supposed to be and where should
    /// it be located.
    pub fn split(&self, opts: &SplitOpts, cmd: &str) -> Pane {
        let pane_id = exec(
            [
                wezterm_cli_vec(),
                vec!["split-pane"],
                self.pane_id_vec(),
                current_dir_vec(&current_dir()),
                opts.as_vec(),
                zsh_c_vec(cmd),
            ]
            .concat(),
        );
        Pane { id: pane_id }
    }
    /// activates the pane identified by `self`, which means, it gets the focus
    pub fn activate(&self) {
        exec([wezterm_cli_vec(), vec!["activate-pane"], self.pane_id_vec()].concat());
    }
}
