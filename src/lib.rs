//! markdown transformer toolkit

use config::Config;
use core::fmt::Debug;
use opts::ParserResult;
use parser::Parser;
use process::chain::Chain;
use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;
use syntax::Tree;

use crate::process::{canonicalize, chain, code_block, image_converter};

pub mod config;
pub mod log;
pub mod opts;
pub mod parser;
pub mod process;

pub mod render;
pub mod syntax;
pub mod term_cli;
pub mod watch;
pub mod wezterm_cli;

pub fn establish_log_level(parsed_opts: &ParserResult) -> u8 {
    if parsed_opts.get_flag("verbose") {
        1
    } else if parsed_opts.get_flag("debug") {
        2
    } else if parsed_opts.get_flag("trace") {
        3
    } else {
        0
    }
}

fn read_all_from_stdin() -> String {
    let lines = io::stdin().lines();
    let mut result = String::new();
    for line in lines {
        result.push_str(line.unwrap().as_str());
        result.push('\n');
    }
    result
}

pub fn read_input(source_file: &str) -> String {
    let start = Instant::now();
    let input = if source_file.is_empty() {
        read_all_from_stdin()
    } else {
        std::fs::read_to_string(source_file).unwrap()
    };
    m_dbg!("input read in:\t\t{:?}", start.elapsed());
    input
}

pub fn create_default_chain() -> Chain {
    m_trc!("constructing chain...");
    let chain = chain::new(canonicalize::new(), image_converter::new()).append(code_block::new());
    m_trc!("done");
    m_dbg!("chain: {:?}", chain);
    chain
}

const MATO_CONFIG_DIR_NAME: &str = "mato";

/// locates and reads a preamble file.
///
/// algorithm for locating:
///
/// 1. a *sibbling* file, located side-by-side the input file is searched, named `name`
/// 2. if not found, a user-wide configuration under $XDG_CONFIG_HOME/mato/`name` is searched
/// 3. if not found, the `default_preamble` string is returned
pub fn locate_and_load_preamble(config: &Config, name: &str, default_preamble: &str) -> String {
    if config.skip_preamble {
        return "".to_string();
    }
    let sibbling_preamble = crate::parent_dir(&config.source_file).join(name);
    if sibbling_preamble.as_path().is_file() {
        m_dbg!("found sibbling preamble: {}", sibbling_preamble.display());
        fs::read_to_string(sibbling_preamble).unwrap()
    } else {
        // no sibbling preamble
        // 1. try XDG_CONFIG_HOME
        let config_home = match env::var("XDG_CONFIG_HOME") {
            Ok(xdg_config_home) => {
                m_dbg!("XDG_CONFIG_HOME = {}", xdg_config_home);
                xdg_config_home
            }
            Err(_) => {
                m_dbg!("XDG_CONFIG_HOME not set");
                // 2. try $HOME/.config
                match env::var("HOME") {
                    Ok(home_path) => format!("{}/.config", home_path),
                    Err(_) => "".to_string(),
                }
            }
        };
        if !config_home.is_empty() {
            // construct mato config path
            let mato_config_path = Path::new(&config_home).join(MATO_CONFIG_DIR_NAME);
            if mato_config_path.exists() {
                m_dbg!("found mato config path {:?}: ", mato_config_path);
                let user_peamble_path = mato_config_path.join(name);
                if user_peamble_path.exists() {
                    m_dbg!("found user preamble: {:?}", user_peamble_path);
                    return fs::read_to_string(user_peamble_path).unwrap();
                }
            } else {
                m_dbg!("mato config path not found: {:?}", mato_config_path);
            }
        }
        m_dbg!("preamble:\t\tbuilt-in");
        default_preamble.to_owned()
    }
}

pub fn parent_dir(file_name: &str) -> &Path {
    Path::new(file_name).parent().unwrap()
}

pub fn replace_file_extension(file_name: &str, extension: &str) -> PathBuf {
    let path_source_file = Path::new(&file_name);
    let mut path_target_file = path_source_file.to_path_buf();
    path_target_file.set_extension(extension);
    path_target_file
}

pub fn create_if_not_exists(file_name: &str) {
    let path_source_file = Path::new(file_name);
    if !path_source_file.is_file() {
        m_dbg!("creating {}", file_name);
        File::create(file_name).unwrap();
    }
}

const EMPTY_PDF: &[u8] = include_bytes!("empty.pdf");

pub fn create_empty_if_not_exists(file_name: &str) {
    let path_source_file = Path::new(file_name);
    if !path_source_file.is_file() {
        m_dbg!("creating empty pdf {}", file_name);
        let mut pdf = File::create(file_name).unwrap();
        pdf.write_all(EMPTY_PDF).unwrap();
    }
}

/// executes the given `cmd` as a sub process, blocks and
/// returns its output as a string
fn spawn(cmd: Vec<&str>) {
    m_dbg!("exec: {:?}", cmd);
    Command::new("/usr/bin/env")
        .args(cmd)
        .status()
        .expect("error executing spawn command");
}

/// executes the given `cmd` as a sub process, blocks and
/// returns its output as a string
fn exec(cmd: Vec<&str>) -> String {
    m_dbg!("exec: {:?}", cmd);
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

/// top-level helper method to transform a given input string into a target language specified by the passed in renderer
pub fn transform<R: Render, P: Process>(
    r: &mut R,
    p: &mut P,
    config: &Config,
    input: &str,
) -> String {
    m_trc!("parsing...");
    let mut tree = Parser::parse(input);
    m_trc!("parsed: {:?}", tree);
    tree = process(p, tree, config);
    m_trc!("{:?}", config);
    if config.dump_dot_file {
        let path_target_file = replace_file_extension(&config.source_file, "dot");
        m_trc!("dumping processed tree to: {:?}", path_target_file);
        fs::write(path_target_file, format!("{}", tree)).expect("Unable to write groff file");
    } else {
        m_trc!("processed:\n{:?}", tree);
    }
    render(r, tree, p)
}

/// A processor processes the AST in some way
pub trait Process: Debug {
    fn process(&mut self, exp: Tree, config: &Config) -> Tree;
}

/// helper function for static dispatch
///
/// calls the passed in processor on the given exp
fn process<P: Process>(p: &mut P, exp: Tree, config: &Config) -> Tree {
    p.process(exp, config)
}

/// A renderer renders an Exp into a String
pub trait Render {
    /// render the passed-in expression into a string
    fn render(&mut self, tree: Tree) -> String;
}

/// helper function for static dispatch
///
/// calls the passed in renderer on the result created by the parser
fn render<R: Render, P: Process>(r: &mut R, exp: Tree, _p: &mut P) -> String {
    r.render(exp)
}

pub fn grotopdf(config: &Config, input: &str) -> Vec<u8> {
    // calling `groff` directly instead of `mompdf` has a performance
    // adavantage, but will handle forwar references not correctly.
    // see https://www.schaffter.ca/mom/pdf/mom-pdf.pdf and there
    // section 6.1
    // I switched to `groff` as `pdfmom` would always call `groff`
    // three times, even when it is not necessary, because the document
    // being processed does not contain any references.
    let mut child = Command::new("/usr/bin/env")
        .arg("groff")
        .arg("-Tpdf")
        .arg("-mom")
        .arg(format!("-m{}", config.lang))
        .args(["-K", "UTF-8"]) // process with preconv to support utf-8
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn pdfmom");

    {
        // this lexical block is only here to let stdin run out of scope to be closed...
        let mut stdin = child.stdin.take().expect("Failed to open stdin for pdfmom");
        stdin
            .write_all(input.as_bytes())
            .expect("Failed to write to stdin of pdfmom");
    }
    // ... otherwise this call would not terminate
    let output = child.wait_with_output().expect("Failed to read stdout");
    if !output.stderr.is_empty() {
        let _ = io::stderr().write(&output.stderr);
    }
    output.stdout
}
