use crate::syntax::Exp;

use super::Render;

#[derive(Default)]
pub struct Renderer {
    char_index: usize,
}

impl Renderer {
    pub fn new() -> Self {
        Self { char_index: 0 }
    }

    fn wrap_at(&mut self, s: String, col: usize) -> String {
        let mut result = String::new();
        if s.starts_with(' ') {
            result = " ".to_string();
            self.char_index += 1;
        }
        for word in s.split(&[' ', '\n']) {
            if self.char_index + word.len() < col {
                if !result.is_empty() && result != " " {
                    result = format!("{} {}", result, word);
                } else {
                    result = format!("{}{}", result, word);
                }
                self.char_index += word.len();
            } else {
                result = format!("{}\n{}", result, word);
                self.char_index = word.len();
            }
            self.char_index += 1;
        }
        result
    }
}

impl Render for Renderer {
    fn render(
        &mut self,
        exp: crate::syntax::Exp,
        ctx: std::collections::HashMap<String, String>,
    ) -> String {
        match exp {
            Exp::Literal(s) => self.wrap_at(s, 68),
            Exp::EscapeLit(s) => s,
            Exp::PreformattedLiteral(s) => s,
            Exp::Bold(b_exp) => {
                let mut bold_text = self.render(*b_exp, ctx);
                // if the text between the * chars would immediately
                // start with a newline, we break the opening * onto
                // the newline instead.
                if bold_text.starts_with('\n') {
                    bold_text.remove(0);
                    self.char_index += 1;
                    format!("\n*{}*", bold_text)
                } else {
                    format!("*{}*", bold_text)
                }
            }
            Exp::Italic(b_exp) => format!("_{}_", self.render(*b_exp, ctx)),
            Exp::CodeBlock(b1, b2) => format!(
                "```{}\n{}```",
                self.render(*b1, ctx.clone()),
                self.render(*b2, ctx)
            ),
            Exp::InlineCode(b_exp) => format!("`{}`", self.render(*b_exp, ctx)),
            Exp::Heading(b_exp, level) => {
                let prefix = (0..level + 1).map(|_| "#").collect::<String>();
                format!("{} {}", prefix, self.render(*b_exp, ctx))
            }
            Exp::Quote(b_exp) => format!("\"{}\"", self.render(*b_exp, ctx)),
            Exp::ChapterMark(b_exp) => format!(">>({})", self.render(*b_exp, ctx)),
            Exp::RightSidenote(b_exp) => format!(">({})", self.render(*b_exp, ctx)),
            Exp::Footnote(b_exp) => format!("^({})", self.render(*b_exp, ctx)),
            Exp::HyperRef(b1, b2) => format!(
                "[{}]({})",
                self.render(*b1, ctx.clone()),
                self.render(*b2, ctx)
            ),
            Exp::Cat(b1, b2) => {
                format!("{}{}", self.render(*b1, ctx.clone()), self.render(*b2, ctx))
            }
            Exp::Empty() => String::new(),
            Exp::Paragraph() => {
                self.char_index = 0;
                "\n".to_string()
            }
            Exp::LineBreak() => {
                self.char_index = 0;
                "\n".to_string()
            }
            Exp::Document() => String::new(),
            Exp::List(b_exp, _) => self.render(*b_exp, ctx),
            Exp::ListItem(b_exp, level) => {
                let indent = (0..level).map(|_| "  ").collect::<String>();
                format!("{}{}", indent, self.render(*b_exp, ctx))
            }
            Exp::MetaDataBlock(b_exp) => format!("---\n{}---\n\n", self.render(*b_exp, ctx)),
            Exp::MetaDataItem(key, value) => format!("{}: {}\n", key, value),
            Exp::Image(b1, b2) => format!(
                "![{}]({})",
                self.render(*b1, ctx.clone()),
                self.render(*b2, ctx)
            ),
            Exp::Color(b_exp) => format!("\\{{{}}}", self.render(*b_exp, ctx)),
        }
    }
}
