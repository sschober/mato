//! tex rendering backend
use std::collections::HashMap;

use crate::Exp;

use super::Render;

/// Renderer renders parsed Exps into TeX strings
pub struct Renderer;

impl Renderer {
    fn render_internal(&mut self, exp: Exp) -> String {
        match exp {
            Exp::Literal(s) | Exp::PreformattedLiteral(s) => s,
            Exp::EscapeLit(s) => format!("\\{s}"),
            Exp::Bold(b_exp) => format!("\\textbf{{{}}}", self.render_internal(*b_exp)),
            Exp::Italic(b_exp) => format!("\\textit{{{}}}", self.render_internal(*b_exp)),
            Exp::CodeBlock(b1, b2) => {
                format!("\\texttt{{{}}}", self.render_internal(*b2))
            }
            Exp::InlineCode(b_exp) => {
                format!("\\texttt{{{}}}", self.render_internal(*b_exp))
            }
            Exp::Heading(b_exp, level) => {
                let section = match level {
                    2 => "subsubsection",
                    1 => "subsection",
                    _ => "section",
                };
                format!("\\{}{{{}}}", section, self.render_internal(*b_exp))
            }
            Exp::Quote(b_exp) => format!("\"`{}\"'", self.render_internal(*b_exp)),
            Exp::ChapterMark(b_exp) | Exp::RightSidenote(b_exp) => self.render_internal(*b_exp),
            Exp::Footnote(b_exp) => format!("~\\footnote{{{}}}", self.render_internal(*b_exp)),
            Exp::HyperRef(b_exp1, b_exp2) => format!(
                "\\href{{{}}}{{{}}}",
                self.render_internal(*b_exp2),
                self.render_internal(*b_exp1)
            ),
            Exp::Cat(b_exp1, b_exp2) => format!(
                "{}{}",
                self.render_internal(*b_exp1),
                self.render_internal(*b_exp2)
            ),
            Exp::Empty() | Exp::Paragraph() | Exp::Document() => String::new(),
            Exp::LineBreak() => "\n".to_string(),
            Exp::List(_b_exp, _) => String::new(),
            Exp::ListItem(_, _) => String::new(),
            Exp::MetaDataBlock(_) => String::new(),
            Exp::MetaDataItem(_, _) => String::new(),
            Exp::Image(_, _) => String::new(),
        }
    }
}
impl Render for Renderer {
    fn render(&mut self, exp: Exp, _: HashMap<String, String>) -> String {
        self.render_internal(exp)
    }
}
