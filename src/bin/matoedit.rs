use std::env;

use mato::opts::Opt;
use mato::term_cli::TermCli;
use mato::{opts, print_version};

#[derive(Default)]
struct Config {
    source_file: String,
    lang: String,
}

fn parse_config() -> Option<Config> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        None
    } else {
        let mut result = Config::default();
        for arg in args {
            match arg.as_str() {
                "-len" | "-l en" => result.lang = "en".to_owned(),
                _ => result.source_file = arg,
            }
        }
        Some(result)
    }
}

const VERSION: &str = "0.1.0";
const PROG_NAME: &str = "matoedit";

/// spawns a new wezterm pane in a new tab and opens the
/// passed in file in an editor in said pane.
/// then splits the pane and launches `matopdf` on the file.
/// then waits for a sec to let `matopdf` finish, then
/// proceeds to create a toplevel split pane to the right,
/// wehere `termpdf.py` is launched on the resulting pdf.
fn main() -> std::io::Result<()> {
    let mut p = opts::Parser::new();
    p.add_opt(Opt::Flag {
        short_name: "v".to_owned(),
        long_name: "version".to_owned(),
        description: "Print command version".to_owned(),
    });
    p.add_opt(Opt::Value {
        short_name: "s".to_owned(),
        long_name: "source-file".to_owned(),
        description: "Source file".to_owned(),
    });

    let parsed_opts = p.parse(env::args().collect());

    if parsed_opts.contains_key("version") {
        print_version(PROG_NAME, VERSION);
        return Ok(());
    }

    // ACQUIRE cli handle, panics if not supported
    let term_cli = TermCli::get();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("need a file as argument!");
    }

    let config = parse_config().expect("need source file");
    eprintln!("file to open: {}", config.source_file);

    // we create the source file in any case, so that we can
    // immediately transform it.
    mato::create_if_not_exists(&config.source_file);

    // OPEN editor
    let editor_handle = term_cli.get_active_windows_handle();
    eprintln!("editor handle: {}", editor_handle);

    // we need to figure out the target file name for termpdf to call on
    let target_file_path = mato::replace_file_extension(&config.source_file, "pdf");
    eprintln!("target file: {}", target_file_path.display());

    // CREATE empty pdf if none is there already
    mato::create_empty_if_not_exists(&format!("{}", target_file_path.display()));

    // LAUNCH matopdf
    let mato_handle = term_cli.exec_matopdf(&config.source_file, &config.lang, editor_handle);
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
    term_cli.open_editor(&config.source_file);

    // CLOSE evrything
    term_cli.close(mato_handle);
    term_cli.close(termpdf_handle);

    Ok(())
}
