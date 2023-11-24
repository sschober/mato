use std::collections::HashMap;

use super::Process;

use crate::config::Config;
use crate::syntax::meta_data_block;
use crate::Exp;

/// The Canonicalizer processor removes unneeded AST
/// elements, like empty()s
pub struct Canonicalizer {}

/// descents the complete AST and erazes Empty() nodes
fn erase_empty(exp: Exp) -> Exp {
    match exp {
        Exp::Cat(b_exp1, b_exp2) => match *b_exp1 {
            Exp::Empty() => erase_empty(*b_exp2),
            _ => erase_empty(*b_exp1).cat(erase_empty(*b_exp2)),
        },
        Exp::CodeBlock(b1, b2) => Exp::CodeBlock(b1, Box::new(erase_empty(*b2))),
        Exp::MetaDataBlock(b_exp) => meta_data_block(erase_empty(*b_exp)),
        Exp::ChapterMark(b_exp) => Exp::ChapterMark(Box::new(erase_empty(*b_exp))),
        _ => exp,
    }
}

impl Process for Canonicalizer {
    fn process(&mut self, exp: Exp, _: &Config) -> Exp {
        erase_empty(exp)
    }

    fn get_context(&mut self) -> std::collections::HashMap<String, String> {
        HashMap::new()
    }
}

pub fn new() -> Box<dyn Process> {
    Box::new(Canonicalizer {})
}
