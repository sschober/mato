use std::collections::HashMap;

use super::Process;

use crate::config::Config;
use crate::syntax::{lit, meta_data_block, prelit};
use crate::{log_trc, Tree};

/// The Canonicalizer processor removes unneeded AST
/// elements, like empty()s
#[derive(Debug)]
pub struct Canonicalizer {}

/// descents the complete AST and erazes Empty() nodes
fn erase_empty(exp: Tree) -> Tree {
    match exp {
        Tree::Document(dt, be) => Tree::Document(dt, Box::new(erase_empty(*be))),
        Tree::Cat(b_exp1, b_exp2) => match *b_exp1 {
            Tree::Empty() => erase_empty(*b_exp2),
            _ => erase_empty(*b_exp1).cat(erase_empty(*b_exp2)),
        },
        Tree::CodeBlock(b1, b2) => Tree::CodeBlock(b1, Box::new(erase_empty(*b2))),
        Tree::MetaDataBlock(b_exp) => meta_data_block(erase_empty(*b_exp)),
        Tree::ChapterMark(b_exp) => Tree::ChapterMark(Box::new(erase_empty(*b_exp))),
        Tree::PreformattedLiteral(s) => prelit(&prelit_escape_groff_symbols(s)),
        Tree::Footnote(be) => Tree::Footnote(Box::new(erase_empty(*be))),
        // the next rule replaces old style numerals in text body literals, 
        // but not in literals in headings
        Tree::Literal(s) => lit(replace_old_style_figures(s).as_ref()),
        
        Tree::SmallCaps(be) => {
            Tree::SmallCaps(Box::new(match *be {
                Tree::Literal(s) => lit(&replace_small_caps(s)),
                _ => *be
            }))
        },
        _ => exp,
    }
}

fn replace_small_caps(s: String) -> String {
    let mut result = String::new();
    for c in s.chars() {
        if c.is_ascii_alphabetic() {
            result.push_str(&format!("\\[{}.sc]", c));
        } else {
            result.push(c);
        }
    }
    result
}

fn replace_old_style_figures(s: String) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '0' => result.push_str("\\[zero.oldstyle]"),
            '1' => result.push_str("\\[one.oldstyle]"),
            '2' => result.push_str("\\[two.oldstyle]"),
            '3' => result.push_str("\\[three.oldstyle]"),
            '4' => result.push_str("\\[four.oldstyle]"),
            '5' => result.push_str("\\[five.oldstyle]"),
            '6' => result.push_str("\\[six.oldstyle]"),
            '7' => result.push_str("\\[seven.oldstyle]"),
            '8' => result.push_str("\\[eight.oldstyle]"),
            '9' => result.push_str("\\[nine.oldstyle]"),
            _ => result.push(c),
        }
    }
    result
}

fn prelit_escape_groff_symbols(s: String) -> String {
    s.replace('\\', "\\\\").replace("^", "\\[ha]").replace("\n.", "\n\\&.")
}

impl Process for Canonicalizer {
    fn process(&mut self, exp: Tree, config: &Config) -> Tree {
        log_trc!(config, "{:?}", self);
        erase_empty(exp)
    }

    fn get_context(&mut self) -> std::collections::HashMap<String, String> {
        HashMap::new()
    }
}

pub fn new() -> Box<dyn Process> {
    Box::new(Canonicalizer {})
}
