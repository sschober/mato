use crate::{m_trc, Process};

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
#[derive(Clone, Copy, Debug)]
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
    fn walk(&mut self, exp: Tree, fmt: InFormat) -> Box<Tree> {
        Box::new(match exp {
            Tree::Document(dt, be) => Tree::Document(dt, self.walk(*be, fmt)),
            Tree::Cat(b_exp1, b_exp2) => match *b_exp1 {
                // this arm erases the empty node and is the actual meat of this processor
                Tree::Empty() => *self.walk(*b_exp2, fmt),
                _ => *self.walk(*b_exp1, fmt).cat_box(self.walk(*b_exp2, fmt)),
            },
            Tree::List(be, lvl) => Tree::List(self.walk(*be, fmt), lvl),
            Tree::ListItem(be, lvl) => {
                let orig = self.replace_numerals;
                self.replace_numerals = false;
                let result = Tree::ListItem(self.walk(*be, fmt), lvl);
                self.replace_numerals = orig;
                result
            }
            Tree::Bold(b_exp) => match *b_exp {
                Tree::Italic(b_inn) => Tree::BoldItalic(self.walk(*b_inn, InFormat::BoldItalic)),
                _ => match fmt {
                    InFormat::Italic => Tree::BoldItalic(self.walk(*b_exp, InFormat::BoldItalic)),
                    _ => Tree::Bold(self.walk(*b_exp, InFormat::Bold)),
                },
            },
            Tree::Italic(b_exp) => match *b_exp {
                Tree::Bold(b_inn) => Tree::BoldItalic(self.walk(*b_inn, InFormat::BoldItalic)),
                _ => match fmt {
                    InFormat::Bold => Tree::BoldItalic(self.walk(*b_exp, InFormat::BoldItalic)),
                    _ => Tree::Italic(self.walk(*b_exp, InFormat::Italic)),
                },
            },
            Tree::CodeBlock(b1, b2) => Tree::CodeBlock(b1, self.walk(*b2, fmt)),
            Tree::MetaDataBlock(b_exp) => meta_data_block(*self.walk(*b_exp, fmt)),
            Tree::ChapterMark(b_exp) => Tree::ChapterMark(self.walk(*b_exp, fmt)),
            Tree::PreformattedLiteral(s) => prelit(&prelit_escape_groff_symbols(s)),
            Tree::Footnote(be) => Tree::Footnote(self.walk(*be, fmt)),
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
    fn process(&mut self, exp: Tree) -> Tree {
        m_trc!("{:?}", self);
        *self.walk(exp, InFormat::None)
    }
}

pub fn new(replace_numerals: bool) -> Box<dyn Process> {
    Box::new(Canonicalizer { replace_numerals })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::{bold, empty, lit, prelit};
    use crate::Tree;

    fn canonicalize(exp: Tree) -> Tree {
        let mut c = Canonicalizer {
            replace_numerals: false,
        };
        c.process(exp)
    }

    fn canonicalize_with_numerals(exp: Tree) -> Tree {
        let mut c = Canonicalizer {
            replace_numerals: true,
        };
        c.process(exp)
    }

    // --- Empty node removal ---

    #[test]
    fn removes_empty_from_cat() {
        // Cat(Empty, Literal) should collapse to just Literal
        let input = empty().cat(lit("hello"));
        let result = canonicalize(input);
        assert_eq!(format!("{result:?}"), "Literal(\"hello\")");
    }

    #[test]
    fn preserves_non_empty_cat() {
        let input = lit("a").cat(lit("b"));
        let result = canonicalize(input);
        assert_eq!(format!("{result:?}"), "Cat(Literal(\"a\"), Literal(\"b\"))");
    }

    // --- Bold/Italic folding into BoldItalic ---

    #[test]
    fn bold_wrapping_italic_becomes_bold_italic() {
        let input = Tree::Bold(Box::new(Tree::Italic(Box::new(lit("text")))));
        let result = canonicalize(input);
        assert_eq!(format!("{result:?}"), "BoldItalic(Literal(\"text\"))");
    }

    #[test]
    fn italic_wrapping_bold_becomes_bold_italic() {
        let input = Tree::Italic(Box::new(Tree::Bold(Box::new(lit("text")))));
        let result = canonicalize(input);
        assert_eq!(format!("{result:?}"), "BoldItalic(Literal(\"text\"))");
    }

    #[test]
    fn bold_inside_italic_context_becomes_bold_italic() {
        // When an outer italic context is tracked, a nested Bold becomes BoldItalic
        let input = Tree::Italic(Box::new(bold(lit("text"))));
        let result = canonicalize(input);
        assert_eq!(format!("{result:?}"), "BoldItalic(Literal(\"text\"))");
    }

    #[test]
    fn plain_bold_stays_bold() {
        let input = bold(lit("text"));
        let result = canonicalize(input);
        assert_eq!(format!("{result:?}"), "Bold(Literal(\"text\"))");
    }

    #[test]
    fn plain_italic_stays_italic() {
        let input = Tree::Italic(Box::new(lit("text")));
        let result = canonicalize(input);
        assert_eq!(format!("{result:?}"), "Italic(Literal(\"text\"))");
    }

    // --- Old-style numeral replacement ---

    #[test]
    fn replaces_digits_with_oldstyle() {
        let input = Tree::Document(
            crate::syntax::DocType::DEFAULT,
            Box::new(lit("abc 123 def")),
        );
        let result = canonicalize_with_numerals(input);
        assert_eq!(
            format!("{result:?}"),
            "Document(DEFAULT, Literal(\"abc \\\\[one.oldstyle]\\\\[two.oldstyle]\\\\[three.oldstyle] def\"))"
        );
    }

    #[test]
    fn numerals_not_replaced_when_disabled() {
        let input = Tree::Document(
            crate::syntax::DocType::DEFAULT,
            Box::new(lit("abc 123")),
        );
        let result = canonicalize(input);
        assert_eq!(format!("{result:?}"), "Document(DEFAULT, Literal(\"abc 123\"))");
    }

    #[test]
    fn numerals_not_replaced_inside_list_items() {
        // replace_numerals is temporarily disabled inside list items
        let item_content = lit("item 42");
        let input = Tree::Document(
            crate::syntax::DocType::DEFAULT,
            Box::new(Tree::List(
                Box::new(Tree::ListItem(Box::new(item_content), 0)),
                0,
            )),
        );
        let result = canonicalize_with_numerals(input);
        // The "42" inside the list item should not be replaced
        assert!(format!("{result:?}").contains("\"item 42\""));
    }

    // --- Small caps replacement ---

    #[test]
    fn small_caps_alphabetic_chars_get_sc_suffix() {
        let input = Tree::SmallCaps(Box::new(lit("Ab")));
        let result = canonicalize(input);
        assert_eq!(
            format!("{result:?}"),
            "SmallCaps(Literal(\"\\\\[A.sc]\\\\[b.sc]\"))"
        );
    }

    #[test]
    fn small_caps_non_alphabetic_chars_unchanged() {
        let input = Tree::SmallCaps(Box::new(lit("1 + 2")));
        let result = canonicalize(input);
        assert_eq!(format!("{result:?}"), "SmallCaps(Literal(\"1 + 2\"))");
    }

    // --- Preformatted literal escaping ---

    #[test]
    fn prelit_backslash_is_doubled() {
        let input = prelit("a\\b");
        let result = canonicalize(input);
        assert_eq!(format!("{result:?}"), "PreformattedLiteral(\"a\\\\\\\\b\")");
    }

    #[test]
    fn prelit_caret_is_escaped() {
        let input = prelit("a^b");
        let result = canonicalize(input);
        assert_eq!(format!("{result:?}"), "PreformattedLiteral(\"a\\\\[ha]b\")");
    }

    #[test]
    fn prelit_dot_at_line_start_is_escaped() {
        let input = prelit("a\n.foo");
        let result = canonicalize(input);
        assert_eq!(
            format!("{result:?}"),
            "PreformattedLiteral(\"a\\n\\\\&.foo\")"
        );
    }
}
