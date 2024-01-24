//! tex rendering backend
use std::collections::HashMap;

use crate::Exp;

use super::Render;

/// Renderer renders parsed Exps into TeX strings
pub struct Renderer;

fn render_internal(exp: Exp) -> String {
    match exp {
        Exp::Literal(s) | Exp::PreformattedLiteral(s) => s,
        Exp::EscapeLit(s) => format!("\\{s}"),
        Exp::Bold(b_exp) => format!("\\textbf{{{}}}", render_internal(*b_exp)),
        Exp::Italic(b_exp) => format!("\\textit{{{}}}", render_internal(*b_exp)),
        Exp::CodeBlock(_b1, b2) => {
            format!("\\texttt{{{}}}", render_internal(*b2))
        }
        Exp::InlineCode(b_exp) => {
            format!("\\texttt{{{}}}", render_internal(*b_exp))
        }
        Exp::Heading(b_exp, level) => {
            let section = match level {
                2 => "subsubsection",
                1 => "subsection",
                _ => "section",
            };
            format!("\\{}{{{}}}", section, render_internal(*b_exp))
        }
        Exp::Quote(b_exp) => format!("\"`{}\"'", render_internal(*b_exp)),
        Exp::ChapterMark(b_exp) | Exp::RightSidenote(b_exp) => render_internal(*b_exp),
        Exp::Footnote(b_exp) => format!("~\\footnote{{{}}}", render_internal(*b_exp)),
        Exp::HyperRef(b_exp1, b_exp2) => format!(
            "\\href{{{}}}{{{}}}",
            render_internal(*b_exp2),
            render_internal(*b_exp1)
        ),
        Exp::Cat(b_exp1, b_exp2) => {
            format!("{}{}", render_internal(*b_exp1), render_internal(*b_exp2))
        }
        Exp::Empty() | Exp::Paragraph() | Exp::Document() => String::new(),
        Exp::LineBreak() => "\n".to_string(),
        Exp::List(_b_exp, _) => String::new(),
        Exp::ListItem(_, _) => String::new(),
        Exp::MetaDataBlock(_) => String::new(),
        Exp::MetaDataItem(_, _) => String::new(),
        Exp::Image(_, _) => String::new(),
        Exp::Color(_) => String::new(),
    }
}
impl Render for Renderer {
    fn render(&mut self, exp: Exp, _: HashMap<String, String>) -> String {
        render_internal(exp)
    }
}
