use crate::Render;

pub struct ManRenderer {
    in_synopsis: bool,
}

pub fn new() -> ManRenderer {
    ManRenderer { in_synopsis: false }
}

impl Render for ManRenderer {
    fn render(&mut self, tree: crate::syntax::Tree) -> String {
        match tree {
            crate::syntax::Tree::Document(_, t) => format!("{}", self.render(*t)),
            crate::syntax::Tree::Paragraph() => {
                if self.in_synopsis {
                    "".to_owned()
                } else {
                    ".P\n".to_owned()
                }
            }
            crate::syntax::Tree::PreformattedLiteral(_) => todo!(),
            crate::syntax::Tree::Literal(l) => l,
            crate::syntax::Tree::EscapeLit(s) => match s.as_str() {
                "." => "\\&.".to_string(),
                _ => s,
            },
            crate::syntax::Tree::DropCap(_, _) => todo!(),
            crate::syntax::Tree::Color(_) => todo!(),
            crate::syntax::Tree::ChapterMark(_) => todo!(),
            crate::syntax::Tree::Heading(t, level, _) => match level {
                0 => format!(".TH {}\n", self.render(*t)),
                1 => {
                    let section_header_name = self.render(*t);
                    if "SYNOPSIS" == section_header_name.to_uppercase() {
                        self.in_synopsis = true;
                    } else {
                        self.in_synopsis = false;
                    };
                    format!(".\n.SH {}\n", section_header_name)
                }
                2 => format!(".SS {}\n", self.render(*t)),
                _ => self.render(*t),
            },
            crate::syntax::Tree::Bold(t) => format!("\\c\n.B {}\\c\n", self.render(*t)),
            crate::syntax::Tree::Italic(t) => format!("\\c\n.I {}\\c\n", self.render(*t)),
            crate::syntax::Tree::BoldItalic(_) => todo!(),
            crate::syntax::Tree::SmallCaps(_) => todo!(),
            crate::syntax::Tree::CodeBlock(_, _) => todo!(),
            crate::syntax::Tree::InlineCode(t) => {
                if self.in_synopsis {
                    format!(".SY {}\n", self.render(*t))
                } else {
                    self.render(*t)
                }
            }
            crate::syntax::Tree::Quote(_) => todo!(),
            crate::syntax::Tree::Footnote(_) => todo!(),
            crate::syntax::Tree::RightSidenote(_) => todo!(),
            crate::syntax::Tree::HyperRef(_, _) => todo!(),
            crate::syntax::Tree::DocRef(_, _) => todo!(),
            crate::syntax::Tree::Cat(t1, t2) => format!("{}{}", self.render(*t1), self.render(*t2)),
            crate::syntax::Tree::List(_, _) => todo!(),
            crate::syntax::Tree::ListItem(_, _) => todo!(),
            crate::syntax::Tree::MetaDataBlock(_) => todo!(),
            crate::syntax::Tree::MetaDataItem(_, _) => todo!(),
            crate::syntax::Tree::ImageSizeSpec(_, _) => todo!(),
            crate::syntax::Tree::Image(_, _, _) => todo!(),
            crate::syntax::Tree::LineBreak() => {
                if self.in_synopsis {
                    "".to_owned()
                } else {
                    "\n".to_owned()
                }
            }
            crate::syntax::Tree::VSpace() => "".to_owned(),
            crate::syntax::Tree::Empty() => "".to_owned(),
        }
    }
}
