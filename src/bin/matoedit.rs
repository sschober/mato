use mato::wezterm_cli::WTCli;
use std::{env, thread, time};

const DEFAULT_EDITOR: &str = "nvim";

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

    // we create the source file in any case, so that we can
    // immediately transform it.
    mato::create_if_not_exists(source_file);

    // we look up the users preferred editor via the environment
    // variable.
    let editor_cmd = match env::var("EDITOR") {
        Ok(val) => val,
        Err(_) => DEFAULT_EDITOR.to_string(),
    };

    let wt_cli = WTCli::new();
    // SPAWN the EDITOR pane!
    let editor_pane = wt_cli.spawn(&format!("{} {}", editor_cmd, source_file));
    eprintln!("editor pane id: {}", editor_pane.id);

    // SPLIT the pane and LAUNCH matopdf
    let mato_pane = editor_pane
        .split(format!("matopdf -w -v {}", source_file).as_str())
        .percent(10)
        .bottom()
        .exec();
    eprintln!("mato pane id: {}", mato_pane.id);

    // WAIT a sec
    // this is ugly, but we need to sleep 1 sec to give
    // matopdf time to transform the source file,
    // otherwise termpdf would bail out
    let one_sec = time::Duration::from_secs(1);
    thread::sleep(one_sec);

    // we need to figure out the target file name for termpdf to call on
    let target_file_path = mato::replace_file_extension(source_file, "pdf");
    eprintln!("target file: {}", target_file_path.display());

    // SPLIT the pane on the top-level and LAUNCH `termpdf.py``
    let termpdf_pane = editor_pane
        .split(&format!("termpdf.py {}", target_file_path.display()))
        .top_level()
        .right()
        .exec();
    eprintln!("termpdf pane id: {}", termpdf_pane.id);

    // FOCUS the EDITOR
    // split and spawn move focus to the newly created panes,
    // so we need to refocus on the editor
    editor_pane.activate();

    Ok(())
}
