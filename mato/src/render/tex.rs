//! tex rendering backend
use crate::Exp;

use super::Render;

/// TexRenderer renders parsed Exps into TeX strings
pub struct Renderer;

impl Render for Renderer {
    fn render(&self, exp: Exp) -> String {
        match exp {
            Exp::Paragraph() => String::new(),
            Exp::Literal(s) => s,
            Exp::EscapeLit(s) => format!("\\{}", s),
            Exp::Bold(b_exp) => format!("\\textbf{{{}}}", self.render(*b_exp)),
            Exp::Italic(b_exp) => format!("\\textit{{{}}}", self.render(*b_exp)),
            Exp::CodeBlock(b_exp) => format!("\\texttt{{{}}}", self.render(*b_exp)),
            Exp::InlineCode(b_exp) => format!("\\texttt{{{}}}", self.render(*b_exp)),
            Exp::Heading(b_exp, level) => {
                let section = match level {
                    2 => "subsubsection",
                    1 => "subsection",
                    _ => "section",
                };
                format!("\\{}{{{}}}", section, self.render(*b_exp))
            }
            Exp::Quote(b_exp) => format!("\"`{}\"'", self.render(*b_exp)),
            Exp::ChapterMark(b_exp) => self.render(*b_exp),
            Exp::RightSidenote(b_exp) => self.render(*b_exp),
            Exp::Footnote(b_exp) => format!("~\\footnote{{{}}}", self.render(*b_exp)),
            Exp::HyperRef(b_exp1, b_exp2) => format!(
                "\\href{{{}}}{{{}}}",
                self.render(*b_exp2),
                self.render(*b_exp1)
            ),
            Exp::Cat(b_exp1, b_exp2) => format!("{}{}", self.render(*b_exp1), self.render(*b_exp2)),
            Exp::Empty() => String::new(),
        }
    }
}
