use std::{env, process::Command};

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
/// user's settings.
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
    pub id: String,
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
    pub fn split(&self, opts: Vec<&str>, cmd: &str) -> Pane {
        let pane_id = exec(
            [
                wezterm_cli_vec(),
                vec!["split-pane"],
                self.pane_id_vec(),
                current_dir_vec(&current_dir()),
                opts,
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
