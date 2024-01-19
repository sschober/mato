use std::env;

use mato::term_cli::TermCli;

/// spawns a new wezterm pane in a new tab and opens the
/// passed in file in and editor in said pane.
/// then splits the pane and launches `matopdf` on the file.
/// then waits for a sec to let `matopdf` finish, then
/// proceeds to create a toplevel split pane to the right,
/// wehere `termpdf.py` is launched on the resulting pdf.
fn main() -> std::io::Result<()> {
    // ACQUIRE cli handle, panics if not supported
    let term_cli = TermCli::get();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("need a file as argument!");
    }

    let source_file = args.get(1).unwrap();
    eprintln!("file to open: {}", source_file);

    // we create the source file in any case, so that we can
    // immediately transform it.
    mato::create_if_not_exists(source_file);

    // OPEN editor
    let editor_handle = term_cli.get_active_windows_handle();
    eprintln!("editor handle: {}", editor_handle);

    // we need to figure out the target file name for termpdf to call on
    let target_file_path = mato::replace_file_extension(source_file, "pdf");
    eprintln!("target file: {}", target_file_path.display());

    // CREATE empty pdf if none is there already
    mato::create_empty_if_not_exists(&format!("{}", target_file_path.display()));

    // LAUNCH matopdf
    let mato_handle = term_cli.exec_matopdf(source_file, editor_handle);
    eprintln!("mato handle: {}", mato_handle);

    // LAUNCH `termpdf.py`
    let termpdf_handle =
        term_cli.exec_termpdf(&format!("{}", target_file_path.display()), editor_handle);
    eprintln!("termpdf handle: {}", termpdf_handle);

    // FOCUS the EDITOR
    // split and spawn move focus to the newly created panes,
    // so we need to refocus on the editor
    term_cli.focus(editor_handle);

    // OPEN editor and block on call
    term_cli.open_editor(source_file);

    // CLOSE evrything
    term_cli.close(mato_handle);
    term_cli.close(termpdf_handle);

    Ok(())
}
