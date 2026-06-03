use std::io::Write;
use std::{
    io,
    process::{Command, Stdio},
};

use crate::syntax::{lit, Tree};
use crate::{m_dbg, m_trc, Process};

/// CodeBlock processor looks inside code blocks that it finds in the AST and
/// if the type is pic will render the pic picture embedded inside of the block.
#[derive(Default, Debug)]
pub struct CodeBlockProcessor {}

fn walk(exp: Tree) -> Tree {
    match exp {
        Tree::Document(dt, be) => Tree::Document(dt, Box::new(walk(*be))),
        Tree::Cat(b1, b2) => walk(*b1).cat(walk(*b2)),
        Tree::CodeBlock(block_type, content) => {
            let match_ref = block_type.as_ref();
            match match_ref {
                Tree::Literal(type_string) => {
                    m_dbg!("processing code block of type {}", type_string);
                    if type_string == "pic" {
                        // process pic contents by piping it through pic
                        process_pic(*content)
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

fn process_pic(content: Tree) -> Tree {
    let mut child = Command::new("/usr/bin/env")
        .arg("pic")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| crate::die!("failed to spawn pic: {e}"));
    let code_block_contents = match content {
        Tree::PreformattedLiteral(value) => value,
        Tree::Literal(value) => value,
        _ => "".to_string(),
    };
    let pic_input = format!(".PS\n{code_block_contents}\n.PE\n");
    {
        // this lexical block is only here to let stdin run out of scope to be closed...
        let mut stdin = child.stdin.take()
            .unwrap_or_else(|| crate::die!("failed to open stdin for pic"));
        stdin
            .write_all(pic_input.as_bytes())
            .unwrap_or_else(|e| crate::die!("failed to write to pic stdin: {e}"));
    }
    // ... otherwise this call would not terminate
    let output = child.wait_with_output()
        .unwrap_or_else(|e| crate::die!("failed to read pic output: {e}"));
    if !output.stderr.is_empty() {
        let _ = io::stderr().write(&output.stderr);
    }
    let rendered_pic = String::from_utf8(output.stdout)
        .unwrap_or_else(|e| crate::die!("pic output is not valid UTF-8: {e}"));
    m_trc!("rendered: {}", rendered_pic);
    lit(&rendered_pic)
}

impl Process for CodeBlockProcessor {
    fn process(&mut self, exp: crate::syntax::Tree) -> crate::syntax::Tree {
        m_trc!("{:?}", self);
        walk(exp)
    }
}

pub fn new() -> Box<dyn Process> {
    Box::new(CodeBlockProcessor {})
}
