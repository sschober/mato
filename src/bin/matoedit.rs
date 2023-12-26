use std::{env, fs::File, path::Path, process::Command};
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
fn current_dir() -> String {
    env::current_dir()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_string()
}
fn spawn(cmd: &str) -> String {
    exec(vec![
        "wezterm",
        "cli",
        "spawn",
        "--cwd",
        current_dir().as_str(),
        "zsh",
        "-c",
        cmd,
    ])
}
fn split_pane_id(id: &str, opts: Vec<&str>, cmd: &str) -> String {
    exec(
        [
            vec![
                "wezterm",
                "cli",
                "split-pane",
                "--pane-id",
                id,
                "--cwd",
                current_dir().as_str(),
            ],
            opts,
            zsh_c_vec(cmd),
        ]
        .concat(),
    )
}
fn zsh_c_vec(cmd: &str) -> Vec<&str> {
    vec!["zsh", "-c", cmd]
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    eprintln!("num args {}", args.len());
    if args.len() != 2 {
        panic!("need a file as argument!");
    }
    let source_file = args.get(1).unwrap();
    eprintln!("file to open: {}", source_file);
    let path_source_file = Path::new(source_file);
    if !path_source_file.is_file() {
        eprintln!("creating {}", source_file);
        File::create(source_file).unwrap();
    }
    let mut path_target_file = path_source_file.to_path_buf();
    path_target_file.set_extension("pdf");
    eprintln!("target file: {}", path_target_file.display());

    let cmd_micro = format!("micro {}", source_file);
    let micro_pane_id = spawn(cmd_micro.as_str());
    eprintln!("{}", micro_pane_id);

    let mato_pane_id = split_pane_id(
        micro_pane_id.as_str(),
        vec!["--percent", "10", "--bottom"],
        format!("matopdf -w -v {}", source_file).as_str(),
    );
    eprintln!("mato pane id: {}", mato_pane_id);

    Ok(())
}
