use std::env;

use mato::term_cli::TermCli;
use mato::{die, establish_log_level, mato_dbg, opt_val, opts};

const VERSION: &str = "0.1.1";
const PROG_NAME: &str = "matoedit";

/// spawns a new wezterm pane in a new tab and opens the
/// passed in file in an editor in said pane.
/// then splits the pane and launches `matopdf` on the file.
/// then waits for a sec to let `matopdf` finish, then
/// proceeds to create a toplevel split pane to the right,
/// wehere `termpdf.py` is launched on the resulting pdf.
fn main() -> std::io::Result<()> {
    let mut p = opts::Parser::new();
    p.add_opt(opt_val!("l", "lang", "Language of source file.", "den"));

    let parsed_opts = p.parse(env::args().collect());
    // eprintln!("{:?}", parsed_opts);

    // handle version and help
    parsed_opts.handle_standard_flags(PROG_NAME, VERSION);

    mato::log::set_log_level(establish_log_level(&parsed_opts));

    let lang = parsed_opts.get_opt("lang");
    // eprintln!("language: {}", lang);

    let mut args: Vec<String> = parsed_opts.params.clone();
    if args.len() != 1 {
        die!("need a file as argument!");
    }

    let source_file = args.pop().unwrap();
    mato_dbg!("source file: {}", source_file);

    // ACQUIRE cli handle, panics if not supported
    let term_cli = TermCli::get();

    // we create the source file in any case, so that we can
    // immediately transform it.
    mato::create_if_not_exists(&source_file);

    // OPEN editor
    let editor_handle = term_cli.get_active_windows_handle();
    mato_dbg!("editor handle: {}", editor_handle);

    // we need to figure out the target file name for termpdf to call on
    let target_file_path = mato::replace_file_extension(&source_file, "pdf");
    mato_dbg!("target file: {}", target_file_path.display());

    // CREATE empty pdf if none is there already
    mato::create_empty_if_not_exists(&format!("{}", target_file_path.display()));

    // LAUNCH matopdf
    let mato_handle = term_cli.exec_matopdf(&source_file, &lang, editor_handle);
    mato_dbg!("mato handle: {}", mato_handle);

    // LAUNCH `termpdf.py`
    let termpdf_handle =
        term_cli.exec_termpdf(&format!("{}", target_file_path.display()), editor_handle);
    mato_dbg!("termpdf handle: {}", termpdf_handle);

    // FOCUS the EDITOR
    // split and spawn move focus to the newly created panes,
    // so we need to refocus on the editor
    term_cli.focus(editor_handle);

    // OPEN editor and block on call
    term_cli.open_editor(&source_file);

    // CLOSE evrything
    term_cli.close(mato_handle);
    term_cli.close(termpdf_handle);

    Ok(())
}
