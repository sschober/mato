use mato::wezterm_cli;
use std::{env, thread, time};

/// spawns a new wezterm pane in a new tab and opens the
/// passed in file in and editor in said pane.
/// then splits the pane and launches `matopdf` on the file.
/// then waits for a sec to let `matopdf` finish, then 
/// proceeds to create a toplevel split pane to the right,
/// wehere `termpdf.py` is launched on the resulting pdf.
fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("need a file as argument!");
    }

    let source_file = args.get(1).unwrap();
    eprintln!("file to open: {}", source_file);

    mato::create_if_not_exists(source_file);

    let editor_cmd = match env::var("EDITOR") {
        Ok(val) => val,
        Err(_) => "nvim".to_string(),
    };

    // SPAWN the EDITOR pane!
    let editor_pane = wezterm_cli::spawn(&format!("{} {}", editor_cmd, source_file));
    eprintln!("editor pane id: {}", editor_pane.id);

    // SPLIT the pane and LAUNCH matopdf
    let mato_pane = editor_pane.split(
        vec!["--percent", "10", "--bottom"],
        format!("matopdf -w -v {}", source_file).as_str(),
    );
    eprintln!("mato pane id: {}", mato_pane.id);

    // WAIT a sec
    // this is ugly, but we need to sleep 1 sec to give
    // matopdf time to transform the source file,
    // otherwise termpdf would bail out
    let one_sec = time::Duration::from_secs(1);
    thread::sleep(one_sec);

    let target_file_path = mato::replace_file_extension(source_file, "pdf");
    eprintln!("target file: {}", target_file_path.display());

    // SPLIT the pand toplevel and LAUNCH `termpdf.py``
    let termpdf_pane = editor_pane.split(
        vec!["--top-level", "--right"],
        &format!("termpdf.py {}", target_file_path.display()),
    );
    eprintln!("termpdf pane id: {}", termpdf_pane.id);

    // FOCUS the EDITOR
    // split and spawn move focus to the newly created panes, 
    // so we need to refocus on the editor
    editor_pane.activate();

    Ok(())
}
