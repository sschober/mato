use std::{env, process::Command};

fn current_dir() -> String {
    env::current_dir()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string()
}

fn exec(cmd: Vec<&str>) -> String {
    eprintln!("exec: {:?}", cmd);
    let out = Command::new("/usr/bin/env")
        .args(cmd)
        .output()
        .expect("error executing spawn command")
        .stdout;
    String::from_utf8(out)
        .unwrap()
        .strip_suffix('\n')
        .unwrap()
        .to_string()
}
fn wezterm_cli_vec() -> Vec<&'static str> {
    vec!["wezterm", "cli"]
}
fn zsh_c_vec(cmd: &str) -> Vec<&str> {
    vec!["zsh", "-c", cmd]
}
fn current_dir_vec(dir: &str) -> Vec<&str> {
    vec!["--cwd", dir]
}
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
    fn pane_id_vec(&self) -> Vec<&str> {
        vec!["--pane-id", self.id.as_str()]
    }
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
    pub fn activate(&self) {
        exec([wezterm_cli_vec(), vec!["activate-pane"], self.pane_id_vec()].concat());
    }
}
