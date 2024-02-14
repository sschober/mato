//! markdown transformer toolkit

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;
use core::fmt::Debug;

use config::Config;
use parser::Parser;
use syntax::Tree;
pub mod config;
pub mod parser;
pub mod process;

pub mod render;
pub mod syntax;
pub mod term_cli;
pub mod watch;
pub mod wezterm_cli;

fn read_all_from_stdin() -> String {
    let lines = io::stdin().lines();
    let mut result = String::new();
    for line in lines {
        result.push_str(line.unwrap().as_str());
        result.push('\n');
    }
    result
}

pub fn read_input(config: &Config) -> String {
    let start = Instant::now();
    let input = if config.source_file.is_empty() {
        read_all_from_stdin()
    } else {
        std::fs::read_to_string(&config.source_file).unwrap()
    };
    log_dbg!(config, "input read in:\t\t{:?}", start.elapsed());
    input
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
        eprintln!("creating {}", file_name);
        File::create(file_name).unwrap();
    }
}

const EMPTY_PDF: &[u8] = include_bytes!("empty.pdf");

pub fn create_empty_if_not_exists(file_name: &str) {
    let path_source_file = Path::new(file_name);
    if !path_source_file.is_file() {
        eprintln!("creating empty pdf {}", file_name);
        let mut pdf = File::create(file_name).unwrap();
        pdf.write_all(EMPTY_PDF).unwrap();
    }
}

/// executes the given `cmd` as a sub process, blocks and
/// returns its output as a string
fn spawn(cmd: Vec<&str>) {
    eprintln!("exec: {:?}", cmd);
    Command::new("/usr/bin/env")
        .args(cmd)
        .status()
        .expect("error executing spawn command");
}

/// executes the given `cmd` as a sub process, blocks and
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

/// top-level helper method to transform a given input string into a target language specified by the passed in renderer
pub fn transform<R: Render, P: Process>(
    r: &mut R,
    p: &mut P,
    config: &Config,
    input: &str,
) -> String {
    log_trc!(config, "parsing...");
    let mut exp = Parser::parse(input);
    log_trc!(config, "parsed: {:?}", exp);
    exp = process(p, exp, config);
    log_trc!(config, "processed: {:?}", exp);
    render(r, exp, p)
}

/// A processor processes the AST in some way
pub trait Process : Debug {
    fn process(&mut self, exp: Tree, config: &Config) -> Tree;
    fn get_context(&mut self) -> HashMap<String, String>;
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
    fn render(&mut self, exp: Tree, ctx: HashMap<String, String>) -> String;
}

/// helper function for static dispatch
///
/// calls the passed in renderer on the result created by the parser
fn render<R: Render, P: Process>(r: &mut R, exp: Tree, p: &mut P) -> String {
    r.render(exp, p.get_context())
}

pub fn grotopdf(config: &Config, input: &str) -> Vec<u8> {
    let mut child = Command::new("/usr/bin/env")
        .arg("pdfmom")
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
