use mato::wezterm_cli;
use std::{env, fs::File, path::Path, thread, time};

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
    let micro_pane = wezterm_cli::spawn(cmd_micro.as_str());
    eprintln!("{}", micro_pane.id);

    let mato_pane = micro_pane.split(
        vec!["--percent", "10", "--bottom"],
        format!("matopdf -w -v {}", source_file).as_str(),
    );
    eprintln!("mato pane id: {}", mato_pane.id);

    // this is ugly, but we need to sleep 1 sec to give
    // matopdf time to transform the source file,
    // otherwise termpdf would bail out
    let one_sec = time::Duration::from_secs(1);
    thread::sleep(one_sec);

    let termpdf_cmd = format!("termpdf.py {}", path_target_file.display());
    let termpdf_pane = micro_pane.split(vec!["--top-level", "--right"], &termpdf_cmd);
    eprintln!("termpdf pane id: {}", termpdf_pane.id);
    Ok(())
}
