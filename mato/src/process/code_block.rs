use std::{
    collections::HashMap,
    process::{Command, Stdio}, io,
};
use std::io::Write;

use super::Process;
use crate::{
    config::Config,
    syntax::{lit, Exp},
};

/// ImageConverter processor currently only transforms
/// path information in the image expression.
/// Ultimately, we want it to do conversion and caching.
#[derive(Default)]
pub struct CodeBlockProcessor {}

impl CodeBlockProcessor {
    fn process_code_blocks(&mut self, exp: Exp, config: &Config) -> Exp {
        match exp {
            Exp::Cat(b1, b2) => self
                .process_code_blocks(*b1, config)
                .cat(self.process_code_blocks(*b2, config)),
            Exp::CodeBlock(block_type, content) => {
                let match_copy = block_type.as_ref();
                match match_copy {
                    Exp::Literal(type_string) => {
                        eprintln!("processing code block of type {}", type_string);
                        if type_string == "pic" {
                            // process pic contents by piping it through pic
                            let mut child = Command::new("/usr/bin/env")
                                .arg("pic")
                                .stdin(Stdio::piped())
                                .stdout(Stdio::piped())
                                .stderr(Stdio::piped())
                                .spawn()
                                .expect("Failed to spawn pic");
                            let code_block_contents =
                            match *content {
                                Exp::PreformattedLiteral(value) => value,
                                Exp::Literal(value) => value,
                                _ => "".to_string()
                            };
                            let pic_input = format!(".PS\n{}\n.PE\n", code_block_contents);
                            eprintln!("input: \n{}", pic_input);
                            {
                                // this lexical block is only here to let stdin run out of scope to be closed...
                                let mut stdin = child.stdin.take().expect("Failed to open stdin for pdfmom");
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
                            eprintln!("rendered: {}", rendered_pic);
                            lit(&rendered_pic)
                        } else {
                            Exp::CodeBlock(block_type, content)
                        }
                    }
                    _ => Exp::CodeBlock(block_type, content),
                }
            }
            _ => exp,
        }
    }
}
impl Process for CodeBlockProcessor {
    fn process(
        &mut self,
        exp: crate::syntax::Exp,
        config: &crate::config::Config,
    ) -> crate::syntax::Exp {
        eprintln!("code block proc");
        self.process_code_blocks(exp, config)
    }

    fn get_context(&mut self) -> std::collections::HashMap<String, String> {
        HashMap::new()
    }
}
