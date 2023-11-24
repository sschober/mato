//! markdown transformer toolkit

use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::time::Instant;

use config::Config;
use parser::Parser;
use syntax::Exp;
pub mod config;
pub mod parser;
pub mod process;
pub mod render;
pub mod syntax;
pub mod watch;

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
    log_dbg!(config, "read in:\t\t{:?}", start.elapsed());
    input
}

/// top-level helper method to transform a given input string into a target language specified by the passed in renderer
pub fn transform<R: render::Render, P: process::Process>(
    r: &mut R,
    p: &mut P,
    config: &Config,
    input: &str,
) -> String {
    let mut exp = Parser::parse(input);
    log_trc!(config, "parsed: {:?}", exp);
    exp = process(p, exp, config);
    log_trc!(config, "processed: {:?}", exp);
    render(r, exp, p)
}

/// helper function for static dispatch
///
/// calls the passed in processor on the given exp
fn process<P: process::Process>(p: &mut P, exp: Exp, config: &Config) -> Exp {
    p.process(exp, config)
}

/// helper function for static dispatch
///
/// calls the passed in renderer on the result created by the parser
fn render<R: render::Render, P: process::Process>(r: &mut R, exp: Exp, p: &mut P) -> String {
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
