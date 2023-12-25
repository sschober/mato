//! groff rendering backend
use std::collections::HashMap;

use super::Render;
use crate::Exp;

/// empty struct to attach Renderer implementation on
pub struct Renderer {
    ctx: HashMap<String, String>,
    document_started: bool,
}

pub fn new() -> Renderer {
    Renderer {
        ctx: HashMap::new(),
        document_started: false,
    }
}

impl Renderer {
    fn is_doctype(&self, doc_type: &str) -> bool {
        self.ctx.contains_key("doctype") && self.ctx.get("doctype").unwrap() == doc_type
    }

    /// groff does not support nested formattings, because it has no
    /// stackable way of switching back to the previous style. we
    /// need to emulate this by passing in the parent style as a
    /// parameter, `parent_format`.
    fn render_with_parent_format(&mut self, exp: Exp, parent_format: &str) -> String {
        // abbreviates call to self.render_with_default_format(...)
        macro_rules! rnd {
            ($( $args:expr ), *) => {
                self.render_with_default_format($( $args ),*)
            }
        }

        // abbreviates call to self.render_with_parent_format(...)
        macro_rules! rnd_pf {
            ($( $args:expr ), *) => {
                self.render_with_parent_format($( $args ),*)
            }
        }

        match exp {
            Exp::Document() => {
                // eprintln!("{:?}", self.ctx);
                let mut result = String::new();
                let mut doctype_emitted = false;
                // doctype needs to be first emitted
                if self.ctx.contains_key("doctype") {
                    let value = self.ctx.get("doctype").unwrap();
                    result = format!(
                        "{}.{} {} {} {}\n",
                        result,
                        "doctype".to_uppercase(),
                        value,
                        "HEADER \"\\*[$TITLE]\" \"\" \"\" ",
                        "FOOTER \"\\*[$AUTHOR]\" \"\" \"\\*S[+2]\\*[SLIDE#]\\*S[-2]\"
                        "
                    );
                    //self.ctx.remove("doctype");
                    doctype_emitted = true;
                }

                if self.ctx.contains_key("preamble") {
                    let value = self.ctx.get("preamble").unwrap();
                    result = format!("{}\n{}\n", result, value);
                    self.ctx.remove("preamble");
                }

                for (key, value) in self.ctx.clone().into_iter() {
                    let key = key.replace(' ', "_");
                    if key == "doctype" && doctype_emitted {
                        continue;
                    }
                    result = format!("{}.{} {}\n", result, key.to_uppercase(), value);
                }
                if !self.ctx.is_empty() && !self.ctx.contains_key("pdf title") {
                    result = format!("{}.PDF_TITLE \"\\*[$TITLE]\"\n", result)
                }
                if !self.ctx.is_empty() && !self.is_doctype("CHAPTER") & !self.is_doctype("SLIDES")
                {
                    // if the user gave no meta data block, we
                    // do not emit a .START
                    result = format!("{}\n.START\n", result);
                    self.document_started = true;
                }
                result
            }
            Exp::Paragraph() => "\n.PP".to_string(),
            Exp::LineBreak() => "\n".to_string(),
            Exp::Literal(s) | Exp::PreformattedLiteral(s) => s,
            Exp::EscapeLit(s) => match s.as_str() {
                "." => "\\&.".to_string(),
                _ => s,
            },
            Exp::Bold(b_exp) => {
                format!("\\*[BD]{}\\*[{}]", rnd_pf!(*b_exp, "BD"), parent_format)
            }
            Exp::Italic(b_exp) => {
                format!("\\*[IT]{}\\*[{}]", rnd_pf!(*b_exp, "IT"), parent_format)
            }
            // Currently there seems to be a bug: https://savannah.gnu.org/bugs/index.php?64561
            // Exp::CodeBlock(b_exp) => format!(".QUOTE_STYLE INDENT 1\n.QUOTE\n.CODE\n.BOX OUTLINED black INSET 18p\n{}.BOX OFF\n.QUOTE OFF", self.render(*b_exp)),
            Exp::CodeBlock(_b1, b2) => format!(
                ".QUOTE_STYLE INDENT 1\n.QUOTE\n.CODE\n{}.QUOTE OFF",
                rnd!(*b2)
            ),
            Exp::InlineCode(b_exp) => format!("\\*[CODE]{}\\*[CODE OFF]", rnd!(*b_exp)),
            Exp::Heading(b_exp, level) => {
                if self.is_doctype("CHAPTER") {
                    if level == 0 {
                        if self.document_started {
                            format!(".COLLATE\n.CHAPTER_TITLE \"{}\"\n.START\n", rnd!(*b_exp))
                        } else {
                            self.document_started = true;
                            format!(".CHAPTER_TITLE \"{}\"\n.START\n", rnd!(*b_exp))
                        }
                    } else {
                        format!(
                            ".SPACE -.7v\n.EW 2\n.HEADING {} \"{}\"\n.EW 0\n",
                            level + 2,
                            rnd!(*b_exp)
                        )
                    }
                } else if self.is_doctype("SLIDES") {
                    if level == 0 {
                        if self.document_started {
                            format!(".NEWSLIDE\n.HEADING {} \"{}\"\n", level + 1, rnd!(*b_exp))
                        } else {
                            self.document_started = true;
                            format!(".START\n.HEADING {} \"{}\"\n", level + 1, rnd!(*b_exp))
                        }
                    } else {
                        format!(
                            ".SPACE -.7v\n.EW 2\n.HEADING {} \"{}\"\n.EW 0\n",
                            level + 1,
                            rnd!(*b_exp)
                        )
                    }
                } else {
                    // all other doc types
                    if 3 == level {
                        format!(
                            ".SPACE -1v\n.MN LEFT\n\\!.ALD 1v\n{}\n.MN OFF",
                            rnd!(*b_exp)
                        )
                    } else if 0 == level {
                        format!(
                            ".SPACE -.7v\n.FT B\n.EW 2\n.HEADING {} \"{}\"\n.EW 0\n.FT R\n.DRH\n",
                            level + 1,
                            &rnd!(*b_exp)
                        )
                    } else if 1 == level {
                        format!(
                            ".SPACE -.7v\n.FT B\n.EW 2\n.HEADING {} \"{}\"\n.EW 0\n.FT R\n",
                            level + 1,
                            &rnd!(*b_exp)
                        )
                    } else {
                        format!(
                            ".SPACE -.7v\n.EW 2\n.HEADING {} \"{}\"\n.EW 0\n",
                            level + 1,
                            rnd!(*b_exp)
                        )
                    }
                }
            }
            Exp::ChapterMark(b_exp) => {
                format!(
                    ".MN RIGHT\n.PT_SIZE +48\n.COLOR grey\n{}\n.MN OFF\n",
                    rnd!(*b_exp)
                )
            }
            Exp::RightSidenote(b_exp) => {
                format!("\n.MN RIGHT\n.PT_SIZE -2\n{}\n.MN OFF\n", rnd!(*b_exp))
            }
            Exp::Quote(b_exp) => format!("\"{}\"", rnd!(*b_exp)),
            Exp::Footnote(b_exp) => {
                format!("\n.FOOTNOTE\n{}\n.FOOTNOTE END\n", rnd!(*b_exp))
            }
            Exp::HyperRef(b_exp1, b_exp2) => {
                format!(".PDF_WWW_LINK {} \"{}\"", rnd!(*b_exp2), rnd!(*b_exp1))
            }
            Exp::Cat(b_exp1, b_exp2) => {
                format!(
                    "{}{}",
                    rnd_pf!(*b_exp1, parent_format),
                    rnd_pf!(*b_exp2, parent_format)
                )
            }
            Exp::Empty() => String::new(),
            Exp::List(b_exp, _) => {
                format!(".LIST\n.SHIFT_LIST 18p\n{}.LIST OFF\n", rnd!(*b_exp))
            }
            Exp::ListItem(b_exp, _) => match *b_exp {
                Exp::Empty() => String::new(),
                _ => format!(".ITEM\n{}\n", rnd!(*b_exp)),
            },
            Exp::MetaDataBlock(b_exp) => rnd!(*b_exp),
            Exp::MetaDataItem(_, _) => String::new(),
            Exp::Image(b_exp, path) => {
                format!(
                    ".PDF_IMAGE {} 200p 150p CAPTION \"{}\"",
                    rnd!(*path),
                    rnd!(*b_exp)
                )
            }
        }
    }
    fn render_with_default_format(&mut self, exp: Exp) -> String {
        self.render_with_parent_format(exp, "ROM")
    }
}

impl Render for Renderer {
    fn render(&mut self, exp: Exp, ctx: HashMap<String, String>) -> String {
        self.ctx = ctx.clone();
        self.render_with_default_format(exp)
    }
}