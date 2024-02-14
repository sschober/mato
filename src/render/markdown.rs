use crate::syntax::Tree;

use crate::Render;

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
        exp: crate::syntax::Tree,
        ctx: std::collections::HashMap<String, String>,
    ) -> String {
        match exp {
            Tree::Literal(s) => self.wrap_at(s, 68),
            Tree::EscapeLit(s) => s,
            Tree::PreformattedLiteral(s) => s,
            Tree::Bold(b_exp) => {
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
            Tree::Italic(b_exp) => format!("_{}_", self.render(*b_exp, ctx)),
            Tree::SmallCaps(be) => format!("{{{}}}", self.render(*be, ctx)),
            Tree::CodeBlock(b1, b2) => format!(
                "```{}\n{}```",
                self.render(*b1, ctx.clone()),
                self.render(*b2, ctx)
            ),
            Tree::InlineCode(b_exp) => format!("`{}`", self.render(*b_exp, ctx)),
            Tree::Heading(b_exp, level) => {
                let prefix = (0..level + 1).map(|_| "#").collect::<String>();
                format!("{} {}", prefix, self.render(*b_exp, ctx))
            }
            Tree::Quote(b_exp) => format!("\"{}\"", self.render(*b_exp, ctx)),
            Tree::ChapterMark(b_exp) => format!(">>({})", self.render(*b_exp, ctx)),
            Tree::RightSidenote(b_exp) => format!(">({})", self.render(*b_exp, ctx)),
            Tree::Footnote(b_exp) => format!("^({})", self.render(*b_exp, ctx)),
            Tree::HyperRef(b1, b2) => format!(
                "[{}]({})",
                self.render(*b1, ctx.clone()),
                self.render(*b2, ctx)
            ),
            Tree::Cat(b1, b2) => {
                format!("{}{}", self.render(*b1, ctx.clone()), self.render(*b2, ctx))
            }
            Tree::Empty() => String::new(),
            Tree::Paragraph() => {
                self.char_index = 0;
                "\n".to_string()
            }
            Tree::LineBreak() => {
                self.char_index = 0;
                "\n".to_string()
            }
            Tree::Document(_,be) => self.render(*be, ctx),
            Tree::List(b_exp, _) => self.render(*b_exp, ctx),
            Tree::ListItem(b_exp, level) => {
                let indent = (0..level).map(|_| "  ").collect::<String>();
                format!("{}{}", indent, self.render(*b_exp, ctx))
            }
            Tree::MetaDataBlock(b_exp) => format!("---\n{}---\n\n", self.render(*b_exp, ctx)),
            Tree::MetaDataItem(key, value) => format!("{}: {}\n", key, value),
            Tree::Image(b1, b2) => format!(
                "![{}]({})",
                self.render(*b1, ctx.clone()),
                self.render(*b2, ctx)
            ),
            Tree::Color(b_exp) => format!("\\{{{}}}", self.render(*b_exp, ctx)),
        }
    }
}
