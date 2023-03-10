//! groff rendering backend
use crate::Exp;

use super::Renderer;

/// empty struct to attach Renderer implementation on
pub struct GroffRenderer;

impl GroffRenderer {
    /// groff does not support nested formattings, because it has no
    /// stackable way of switching back to the previous style. we
    /// need to emulate this by passing in the parent style as a
    /// parameter, parent_format.
    fn render_with_parent_format(&self, exp: Exp, parent_format: &str) -> String {
        match exp {
            Exp::Literal(s) => s,
            Exp::EscapeLit(s) => {
                match s.as_str() {
                    "." => "\\&.".to_string(),
                    _ => s
                }
            },
            Exp::Bold(b_exp) => {
                format!("\\*[BD]{}\\*[{}]",
                        self.render_with_parent_format(*b_exp, "BD"),
                        parent_format)
            },
            Exp::Italic(b_exp) => {
                format!("\\*[IT]{}\\*[{}]",
                        self.render_with_parent_format(*b_exp, "IT"),
                        parent_format)
            },
            Exp::CodeBlock(b_exp) => format!(".QUOTE_STYLE INDENT 1\n.QUOTE\n.CODE\n.BOX OUTLINED black INSET 18p\n{}.BOX OFF\n.QUOTE OFF", self.render(*b_exp)),
            Exp::InlineCode(b_exp) => format!("\\*[CODE]{}\\*[CODE OFF]", self.render(*b_exp)),
            Exp::Heading(b_exp, level) => {
                format!(".HEADING {} \"{}\"", level + 1, self.render(*b_exp))
            }
            Exp::Quote(b_exp) => format!("\"{}\"", self.render(*b_exp)),
            Exp::Footnote(b_exp) => format!("\n.FOOTNOTE\n{}\n.FOOTNOTE END\n", self.render(*b_exp)),
            Exp::HyperRef(b_exp1, b_exp2) => format!(".PDF_WWW_LINK {} \"{}\"", self.render(*b_exp2), self.render(*b_exp1)),
            Exp::Cat(b_exp1, b_exp2) => {
                format!("{}{}",
                        self.render_with_parent_format(*b_exp1, parent_format),
                        self.render_with_parent_format(*b_exp2, parent_format))
            },
            Exp::Empty() => String::new(),
        }
    }
}

impl Renderer for GroffRenderer {
    fn render(&self, exp: Exp) -> String {
        self.render_with_parent_format(exp, "ROM")
    }
}
