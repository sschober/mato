//! groff rendering backend
use std::collections::HashMap;

use super::Render;
use crate::Exp;

/// empty struct to attach Renderer implementation on
pub struct Renderer {
    ctx: HashMap<String, String>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            ctx: HashMap::new(),
        }
    }

    /// groff does not support nested formattings, because it has no
    /// stackable way of switching back to the previous style. we
    /// need to emulate this by passing in the parent style as a
    /// parameter, `parent_format`.
    fn render_with_parent_format(&mut self, exp: Exp, parent_format: &str) -> String {
        match exp {
            Exp::Document() => {
                eprintln!("{:?}", self.ctx);
                let mut result = String::new();
                for (key, value) in self.ctx.clone().into_iter(){
                    let key = key.replace(' ', "_");
                    result = format!("{}.{} {}\n", result, key.to_uppercase(), value);
                }
                if !self.ctx.is_empty() && !self.ctx.contains_key("pdf title"){
                    result = format!("{}.PDF_TITLE \"*[$TITLE]\"\n", result)
                }
                if !self.ctx.is_empty() {
                    // if the user gave no meta data block, we
                    // do not emit a .START
                    result = format!("{}\n.START\n", result);
                }
                result
            },
            Exp::Paragraph() => "\n.PP".to_string(),
            Exp::Literal(s) => s,
            Exp::EscapeLit(s) => match s.as_str() {
                "." => "\\&.".to_string(),
                _ => s,
            },
            Exp::Bold(b_exp) => {
                format!(
                    "\\*[BD]{}\\*[{}]",
                    self.render_with_parent_format(*b_exp, "BD"),
                    parent_format
                )
            }
            Exp::Italic(b_exp) => {
                format!(
                    "\\*[IT]{}\\*[{}]",
                    self.render_with_parent_format(*b_exp, "IT"),
                    parent_format
                )
            }
            // Currently there seems to be a bug: https://savannah.gnu.org/bugs/index.php?64561
            // Exp::CodeBlock(b_exp) => format!(".QUOTE_STYLE INDENT 1\n.QUOTE\n.CODE\n.BOX OUTLINED black INSET 18p\n{}.BOX OFF\n.QUOTE OFF", self.render(*b_exp)),
            Exp::CodeBlock(b_exp) => format!(
                ".QUOTE_STYLE INDENT 1\n.QUOTE\n.CODE\n{}.QUOTE OFF",
                self.render_with_default_format(*b_exp)
            ),
            Exp::InlineCode(b_exp) => format!(
                "\\*[CODE]{}\\*[CODE OFF]",
                self.render_with_default_format(*b_exp)
            ),
            Exp::Heading(b_exp, level) => {
                if self.ctx.contains_key("doctype") && self.ctx.get("doctype").unwrap() == "CHAPTER" {
                    if level == 0 {
                        format!(".COLLATE\n.CHAPTER_TITLE \"{}\"\n.START\n", self.render_with_default_format(*b_exp))
                    } else {
                        format!(
                            ".SPACE -.7v\n.EW 2\n.HEADING {} \"{}\"\n.EW 0\n",
                            level + 2,
                            self.render_with_default_format(*b_exp)
                        )
                    }
                } else {
                    // all other doc types
                    if 3 == level {
                        format!(
                            ".SPACE -1v\n.MN LEFT\n\\!.ALD 1v\n{}\n.MN OFF",
                            self.render_with_default_format(*b_exp)
                        )
                    } else {
                        format!(
                            ".SPACE -.7v\n.EW 2\n.HEADING {} \"{}\"\n.EW 0\n",
                            level + 1,
                            self.render_with_default_format(*b_exp)
                        )
                    }
                }
            }
            Exp::ChapterMark(b_exp) => {
                format!(
                    ".MN RIGHT\n.PT_SIZE +48\n.COLOR grey\n{}\n.MN OFF\n",
                    self.render_with_default_format(*b_exp)
                )
            }
            Exp::RightSidenote(b_exp) => {
                format!(
                    "\n.MN RIGHT\n.PT_SIZE -2\n{}\n.MN OFF\n",
                    self.render_with_default_format(*b_exp)
                )
            }
            Exp::Quote(b_exp) => format!("\"{}\"", self.render_with_default_format(*b_exp)),
            Exp::Footnote(b_exp) => {
                format!(
                    "\n.FOOTNOTE\n{}\n.FOOTNOTE END\n",
                    self.render_with_default_format(*b_exp)
                )
            }
            Exp::HyperRef(b_exp1, b_exp2) => format!(
                ".PDF_WWW_LINK {} \"{}\"",
                self.render_with_default_format(*b_exp2),
                self.render_with_default_format(*b_exp1)
            ),
            Exp::Cat(b_exp1, b_exp2) => {
                format!(
                    "{}{}",
                    self.render_with_parent_format(*b_exp1, parent_format),
                    self.render_with_parent_format(*b_exp2, parent_format)
                )
            }
            Exp::Empty() => String::new(),
            Exp::List(b_exp, _) => {
                format!(
                    ".LIST\n.SHIFT_LIST 18p\n{}.LIST OFF\n",
                    self.render_with_default_format(*b_exp)
                )
            }
            Exp::ListItem(b_exp, _) => match *b_exp {
                Exp::Empty() => String::new(),
                _ => format!(".ITEM\n{}\n", self.render_with_default_format(*b_exp)),
            },
            Exp::MetaDataBlock(b_exp) => self.render_with_default_format(*b_exp),
            Exp::MetaDataItem(_, _) => {
                String::new()
            }
        }
    }
    fn render_with_default_format(&mut self, exp: Exp) -> String {
        self.render_with_parent_format(exp, "ROM")
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for Renderer {
    fn render(&mut self, exp: Exp, ctx: HashMap<String, String>) -> String {
        self.ctx = ctx.clone();
        self.render_with_default_format(exp)
    }
}
