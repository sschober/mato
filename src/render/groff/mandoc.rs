use crate::Render;

pub struct ManDocRenderer {
    in_title: bool,
    title_open: bool,
}

pub fn new() -> ManDocRenderer {
    ManDocRenderer {
        in_title: true,
        title_open: true,
    }
}

impl Render for ManDocRenderer {
    fn render(&mut self, tree: crate::syntax::Tree) -> String {
        match tree {
            crate::syntax::Tree::Document(_, t) => format!(".Dd $Mdocdate$\n{}", self.render(*t)),
            crate::syntax::Tree::Paragraph() => ".Pp\n".to_owned(),
            crate::syntax::Tree::PreformattedLiteral(_) => todo!(),
            crate::syntax::Tree::Literal(l) => l,
            crate::syntax::Tree::EscapeLit(s) => match s.as_str() {
                "." => "\\&.".to_string(),
                _ => s,
            },
            crate::syntax::Tree::DropCap(c, _) => format!("{}", c as char),
            crate::syntax::Tree::Color(_) => "".to_owned(),
            crate::syntax::Tree::ChapterMark(_) => "".to_owned(),
            crate::syntax::Tree::Heading(t, level, _) => match level {
                0 => {
                    let title = self.render(*t);
                    format!(
                        ".Dt {} 7\n.Os\n.Sh Title\n.Nm {}\n",
                        title.to_uppercase(),
                        title
                    ) // TODO section number configurable
                }
                1 => {
                    let section_header_name = self.render(*t);
                    if self.in_title {
                        // the first level 1 (read ##) header ends the title section
                        self.in_title = false;
                        self.title_open = false;
                        format!(".Nd {}", section_header_name)
                    } else {
                        format!(".Sh {}", section_header_name)
                    }
                }
                2 => format!(".Ss {}", self.render(*t)),
                _ => self.render(*t),
            },
            crate::syntax::Tree::Bold(t) => format!("\\c\n.B {}\\c\n", self.render(*t)),
            crate::syntax::Tree::Italic(t) => {
                let italic_text = self.render(*t);
                format!("\\c\n.I {}\\c\n", italic_text)
            }
            crate::syntax::Tree::BoldItalic(_) => todo!(),
            crate::syntax::Tree::SmallCaps(_) => todo!(),
            crate::syntax::Tree::CodeBlock(_, _) => todo!(),
            crate::syntax::Tree::InlineCode(t) => {
                if self.in_title {
                    let mut sy_closing = "";
                    if self.title_open {
                        sy_closing = ".YS\n";
                    }
                    self.title_open = true;
                    format!("{}.SY {}\n", sy_closing, self.render(*t))
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
            crate::syntax::Tree::List(t, _) => format!(".\n{}", self.render(*t)),
            crate::syntax::Tree::ListItem(t, _) => format!(".TP\n.B {}", self.render(*t)),
            crate::syntax::Tree::MetaDataBlock(_) => todo!(),
            crate::syntax::Tree::MetaDataItem(_, _) => todo!(),
            crate::syntax::Tree::ImageSizeSpec(_, _) => todo!(),
            crate::syntax::Tree::Image(_, _, _) => todo!(),
            crate::syntax::Tree::LineBreak() => {
                if self.in_title {
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
