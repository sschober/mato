use std::io::Write;
use std::{
    io,
    process::{Command, Stdio},
};

use crate::Process;
use crate::{
    config::Config,
    syntax::{lit, Tree},
};
use crate::{log_dbg, log_trc};

/// CodeBlock processor looks inside code blocks that it finds in the AST and
/// if the type is pic will render the pic picture embedded inside of the block.
#[derive(Default, Debug)]
pub struct CodeBlockProcessor {}

fn process_code_blocks(exp: Tree, config: &Config) -> Tree {
    match exp {
        Tree::Cat(b1, b2) => process_code_blocks(*b1, config).cat(process_code_blocks(*b2, config)),
        Tree::CodeBlock(block_type, content) => {
            let match_copy = block_type.as_ref();
            match match_copy {
                Tree::Literal(type_string) => {
                    log_dbg!(config, "processing code block of type {}", type_string);
                    if type_string == "pic" {
                        // process pic contents by piping it through pic
                        let mut child = Command::new("/usr/bin/env")
                            .arg("pic")
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn()
                            .expect("Failed to spawn pic");
                        let code_block_contents = match *content {
                            Tree::PreformattedLiteral(value) => value,
                            Tree::Literal(value) => value,
                            _ => "".to_string(),
                        };
                        let pic_input = format!(".PS\n{}\n.PE\n", code_block_contents);
                        {
                            // this lexical block is only here to let stdin run out of scope to be closed...
                            let mut stdin =
                                child.stdin.take().expect("Failed to open stdin for pdfmom");
                            stdin
                                .write_all(pic_input.as_bytes())
                                .expect("Failed to write to stdin of pdfmom");
                        }
                        // ... otherwise this call would not terminate
                        let output = child.wait_with_output().expect("Failed to read stdout");
                        if !output.stderr.is_empty() {
                            let _ = io::stderr().write(&output.stderr);
                        }
                        let rendered_pic = String::from_utf8(output.stdout).unwrap();
                        log_trc!(config, "rendered: {}", rendered_pic);
                        lit(&rendered_pic)
                    } else {
                        Tree::CodeBlock(block_type, content)
                    }
                }
                _ => Tree::CodeBlock(block_type, content),
            }
        }
        _ => exp,
    }
}
impl Process for CodeBlockProcessor {
    fn process(
        &mut self,
        exp: crate::syntax::Tree,
        config: &crate::config::Config,
    ) -> crate::syntax::Tree {
        log_trc!(config, "{:?}", self);
        process_code_blocks(exp, config)
    }
}

pub fn new() -> Box<dyn Process> {
    Box::new(CodeBlockProcessor {})
}
