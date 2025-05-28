use crate::{m_trc, Process};

use crate::config::Config;
use crate::syntax::{lit, meta_data_block, prelit};
use crate::Tree;

/// The Canonicalizer processor removes unneeded AST
/// elements, like empty()s
#[derive(Debug)]
pub struct Canonicalizer {
    replace_numerals: bool,
}

/// Captures the current font style.
/// We need to remember in which kind of format we are currently in
/// to decided, if we need to inject a Tree::BoldItalic instead of
/// Tree::Bold or Tree::Italic.
#[derive(Clone, Copy)]
enum InFormat {
    Bold,
    Italic,
    BoldItalic,
    None,
}
impl Canonicalizer {
    /// descents the complete AST and
    /// * erazes Empty() nodes
    /// * condenses nested bold and italics style node into single bold-italic style nodes.
    ///   This also works, if the nesting is not direct, but 'far' like **bold and _italic_**
    /// * replaces numerals with old style figures
    /// * replaces Tree::SmallCaps nodes with literal groff .sc characters
    fn erase_empty(&mut self, exp: Tree, fmt: InFormat) -> Box<Tree> {
        Box::new(match exp {
            Tree::Document(dt, be) => Tree::Document(dt, self.erase_empty(*be, fmt)),
            Tree::Cat(b_exp1, b_exp2) => match *b_exp1 {
                // this arm erases the empty node and is the actual meat of this processor
                Tree::Empty() => *self.erase_empty(*b_exp2, fmt),
                _ => *self
                    .erase_empty(*b_exp1, fmt)
                    .cat_box(self.erase_empty(*b_exp2, fmt)),
            },
            Tree::Bold(b_exp) => match *b_exp {
                Tree::Italic(b_inn) => {
                    Tree::BoldItalic(self.erase_empty(*b_inn, InFormat::BoldItalic))
                }
                _ => match fmt {
                    InFormat::Italic => {
                        Tree::BoldItalic(self.erase_empty(*b_exp, InFormat::BoldItalic))
                    }
                    _ => Tree::Bold(self.erase_empty(*b_exp, InFormat::Bold)),
                },
            },
            Tree::Italic(b_exp) => match *b_exp {
                Tree::Bold(b_inn) => {
                    Tree::BoldItalic(self.erase_empty(*b_inn, InFormat::BoldItalic))
                }
                _ => match fmt {
                    InFormat::Bold => {
                        Tree::BoldItalic(self.erase_empty(*b_exp, InFormat::BoldItalic))
                    }
                    _ => Tree::Italic(self.erase_empty(*b_exp, InFormat::Italic)),
                },
            },
            Tree::CodeBlock(b1, b2) => Tree::CodeBlock(b1, self.erase_empty(*b2, fmt)),
            Tree::MetaDataBlock(b_exp) => meta_data_block(*self.erase_empty(*b_exp, fmt)),
            Tree::ChapterMark(b_exp) => Tree::ChapterMark(self.erase_empty(*b_exp, fmt)),
            Tree::PreformattedLiteral(s) => prelit(&prelit_escape_groff_symbols(s)),
            Tree::Footnote(be) => Tree::Footnote(self.erase_empty(*be, fmt)),
            // the next rule replaces old style numerals in text body literals,
            // but not in literals in headings
            Tree::Literal(s) => {
                if self.replace_numerals {
                    lit(replace_old_style_figures(s).as_ref())
                } else {
                    lit(s.as_ref())
                }
            }
            Tree::SmallCaps(be) => Tree::SmallCaps(Box::new(match *be {
                Tree::Literal(s) => lit(&replace_small_caps(s)),
                _ => *be,
            })),
            _ => exp,
        })
    }
}
/// appends a `.sc` to characters
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

/// replaces 0-9 with old style figure references
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
    s.replace('\\', "\\\\")
        .replace('^', "\\[ha]")
        .replace("\n.", "\n\\&.")
}

impl Process for Canonicalizer {
    fn process(&mut self, exp: Tree, _config: &Config) -> Tree {
        m_trc!("{:?}", self);
        *self.erase_empty(exp, InFormat::None)
    }
}

pub fn new(replace_numerals: bool) -> Box<dyn Process> {
    Box::new(Canonicalizer { replace_numerals })
}
