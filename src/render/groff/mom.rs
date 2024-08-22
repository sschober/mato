//! groff rendering backend
use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::{log_dbg, Render};
use crate::{syntax::DocType, Tree};

/// empty struct to attach Renderer implementation on
pub struct Renderer<'a> {
    ctx: HashMap<String, String>,
    document_started: bool,
    doc_type: DocType,
    default_preamble: String,
    config: &'a Config,
}

pub fn new(config: &Config) -> Renderer {
    let default_mom_preamble = include_str!("default-preamble.mom").to_string();

    Renderer {
        ctx: HashMap::new(),
        document_started: false,
        doc_type: DocType::DEFAULT,
        default_preamble: default_mom_preamble,
        config,
    }
}

const PREAMBLE_FILE_NAME: &str = "preamble.mom";

impl Display for DocType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(".DOCTYPE {:?}{}", self, match self {
            DocType::SLIDES => " HEADER \"\\*[$TITLE]\" \"\" \"\" FOOTER \"\\*[$AUTHOR]\" \"\" \"\\*S[+2]\\*[SLIDE#]\\*S[-2]\"",
            DocType::CHAPTER => " HEADER \"\\*[$TITLE]\" \"\" \"\" FOOTER \"\\*[$AUTHOR]\" \"\" \"\"",
            _ =>""
        }))
    }
}

impl Renderer<'_> {
    pub fn locate_and_load_preamble(&mut self, name: &str) -> String {
        if self.config.skip_preamble {
            return "".to_string();
        }
        let sibbling_preamble = Path::new(&self.config.parent_dir).join(name);
        let config = &self.config;
        if sibbling_preamble.as_path().is_file() {
            log_dbg!(
                config,
                "found sibbling preamble: {}",
                sibbling_preamble.display()
            );
            fs::read_to_string(sibbling_preamble).unwrap()
        } else {
            log_dbg!(config, "preamble:\t\tbuilt-in");
            self.default_preamble.to_string()
        }
    }

    /// groff does not support nested formattings, because it has no
    /// stackable way of switching back to the previous style. we
    /// need to emulate this by passing in the parent style as a
    /// parameter, `parent_format`.
    fn render_with_parent_format(&mut self, exp: Tree, parent_format: &str) -> String {
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
            Tree::Document(dt, be) => {
                self.doc_type = dt.clone();
                let mut result = format!("{}", dt);

                if !self.config.skip_preamble {
                    result = format!(
                        "{}\n{}\n",
                        result,
                        self.locate_and_load_preamble(PREAMBLE_FILE_NAME)
                    );
                }

                for (key, value) in self.ctx.clone().into_iter() {
                    let key = key.replace(' ', "_");
                    result = format!("{}.{} {}\n", result, key.to_uppercase(), value);
                }
                if !self.ctx.is_empty() && !self.ctx.contains_key("pdf title") {
                    result = format!("{}.PDF_TITLE \"\\*[$TITLE]\"\n", result)
                }
                match dt {
                    DocType::CHAPTER | DocType::SLIDES => (),
                    _ => {
                        result = format!("{}\n.START\n", result);
                    }
                }
                format!("{}{}", result, rnd_pf!(*be, parent_format))
            }
            Tree::Paragraph() => "\n.PP".to_string(),
            Tree::LineBreak() => "\n".to_string(),
            Tree::Literal(s) | Tree::PreformattedLiteral(s) => s,
            Tree::EscapeLit(s) => match s.as_str() {
                "." => "\\&.".to_string(),
                _ => s,
            },
            Tree::Bold(b_exp) => {
                format!("\\*[BD]{}\\*[{}]", rnd_pf!(*b_exp, "BD"), parent_format)
            }
            Tree::SmallCaps(be) => rnd_pf!(*be, parent_format),
            Tree::Italic(b_exp) => {
                format!("\\*[IT]{}\\*[{}]", rnd_pf!(*b_exp, "IT"), parent_format)
            }
            // Currently there seems to be a bug: https://savannah.gnu.org/bugs/index.php?64561
            // Exp::CodeBlock(b_exp) => format!(".QUOTE_STYLE INDENT 1\n.QUOTE\n.CODE\n.BOX OUTLINED black INSET 18p\n{}.BOX OFF\n.QUOTE OFF", self.render(*b_exp)),
            Tree::CodeBlock(_b1, b2) => format!(
                ".QUOTE_STYLE INDENT 1\n.QUOTE\n.CODE\n{}.QUOTE OFF\n",
                rnd!(*b2)
            ),
            Tree::InlineCode(b_exp) => format!("\\*[CODE]{}\\*[CODE OFF]", rnd!(*b_exp)),
            Tree::Heading(b_exp, level) => {
                match self.doc_type {
                    DocType::CHAPTER => {
                        if level == 0 {
                            format!(
                                "{}.CHAPTER_TITLE \"{}\"\n.START\n",
                                if self.document_started {
                                    ".COLLATE\n"
                                } else {
                                    self.document_started = true;
                                    ""
                                },
                                rnd!(*b_exp)
                            )
                        } else {
                            format!(
                                ".SPACE -.7v\n.EW 2\n.HEADING {} \"{}\"\n.EW 0\n",
                                level + 2,
                                rnd!(*b_exp)
                            )
                        }
                    }
                    DocType::SLIDES => {
                        if level == 0 {
                            format!(
                                ".{}\n.HEADING {} \"{}\"\n",
                                if self.document_started {
                                    "NEWSLIDE"
                                } else {
                                    self.document_started = true;
                                    "START"
                                },
                                level + 1,
                                rnd!(*b_exp)
                            )
                        } else {
                            format!(
                                ".SPACE -.7v\n.EW 2\n.HEADING {} \"{}\"\n.EW 0\n",
                                level + 1,
                                rnd!(*b_exp)
                            )
                        }
                    }
                    _ => {
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
            }
            Tree::Color(b_exp) => {
                format!(".COLOR {}\n", rnd!(*b_exp))
            }
            Tree::ChapterMark(b_exp) => {
                format!(".MN RIGHT\n.PT_SIZE +48\n{}\n.MN OFF\n", rnd!(*b_exp))
            }
            Tree::RightSidenote(b_exp) => {
                format!("\n.MN RIGHT\n.PT_SIZE -2\n{}\n.MN OFF\n", rnd!(*b_exp))
            }
            Tree::Quote(b_exp) => format!("\"{}\"", rnd!(*b_exp)),
            Tree::Footnote(b_exp) => {
                format!("\\c\n.FOOTNOTE\n{}\n.FOOTNOTE END\n", rnd!(*b_exp))
            }
            Tree::HyperRef(b_exp1, b_exp2) => {
                format!(
                    "\\c\n.PDF_WWW_LINK {} \"{}\"\\c\n",
                    rnd!(*b_exp2),
                    rnd!(*b_exp1)
                )
            }
            Tree::Cat(b_exp1, b_exp2) => {
                format!(
                    "{}{}",
                    rnd_pf!(*b_exp1, parent_format),
                    rnd_pf!(*b_exp2, parent_format)
                )
            }
            Tree::Empty() => String::new(),
            Tree::List(b_exp, _) => {
                format!(".LIST\n.SHIFT_LIST 18p\n{}.LIST OFF\n", rnd!(*b_exp))
            }
            Tree::ListItem(b_exp, _) => match *b_exp {
                Tree::Empty() => String::new(),
                _ => format!(".ITEM\n{}\n", rnd!(*b_exp)),
            },
            Tree::MetaDataBlock(b_exp) => rnd!(*b_exp),
            Tree::MetaDataItem(key, value) => {
                format!(".{} {}\n", key.to_uppercase().replace(' ', "_"), value)
            }
            Tree::Image(b_exp, path) => {
                format!(
                    ".PDF_IMAGE {} 200p 150p CAPTION \"{}\"",
                    rnd!(*path),
                    rnd!(*b_exp)
                )
            }
        }
    }
    fn render_with_default_format(&mut self, exp: Tree) -> String {
        self.render_with_parent_format(exp, "ROM")
    }
}

impl Render for Renderer<'_> {
    fn render(&mut self, exp: Tree) -> String {
        self.render_with_default_format(exp)
    }
}
