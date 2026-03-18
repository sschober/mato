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
                    result = format!("{result} {word}");
                } else {
                    result = format!("{result}{word}");
                }
                self.char_index += word.len();
            } else {
                result = format!("{result}\n{word}");
                self.char_index = word.len();
            }
            self.char_index += 1;
        }
        result
    }
}

impl Render for Renderer {
    fn render(&mut self, exp: crate::syntax::Tree) -> String {
        match exp {
            Tree::Literal(s) => self.wrap_at(s, 68),
            Tree::EscapeLit(s) => s,
            Tree::PreformattedLiteral(s) => s,
            Tree::Bold(b_exp) => {
                let mut bold_text = self.render(*b_exp);
                // if the text between the * chars would immediately
                // start with a newline, we break the opening * onto
                // the newline instead.
                if bold_text.starts_with('\n') {
                    bold_text.remove(0);
                    self.char_index += 1;
                    format!("\n*{bold_text}*")
                } else {
                    format!("*{bold_text}*")
                }
            }
            Tree::Italic(b_exp) => format!("_{}_", self.render(*b_exp)),
            Tree::BoldItalic(b_exp) => format!("_**{}**_", self.render(*b_exp)),
            Tree::SmallCaps(be) => format!("{{{}}}", self.render(*be)),
            Tree::CodeBlock(b1, b2) => format!("```{}\n{}```", self.render(*b1), self.render(*b2)),
            Tree::InlineCode(b_exp) => format!("`{}`", self.render(*b_exp)),
            Tree::Heading(b_exp, level, _) => {
                let prefix = (0..level + 1).map(|_| "#").collect::<String>();
                format!("{} {}", prefix, self.render(*b_exp))
            }
            Tree::Quote(b_exp) => format!("\"{}\"", self.render(*b_exp)),
            Tree::ChapterMark(b_exp) => format!(">>({})", self.render(*b_exp)),
            Tree::RightSidenote(b_exp) => format!(">({})", self.render(*b_exp)),
            Tree::Footnote(b_exp) => format!("^({})", self.render(*b_exp)),
            Tree::HyperRef(b1, b2) => format!("[{}]({})", self.render(*b1), self.render(*b2)),
            Tree::Cat(b1, b2) => {
                format!("{}{}", self.render(*b1), self.render(*b2))
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
            Tree::Document(_, be) => self.render(*be),
            Tree::List(b_exp, _) => self.render(*b_exp),
            Tree::ListItem(b_exp, level) => {
                let indent = (0..level).map(|_| "  ").collect::<String>();
                format!("{}{}", indent, self.render(*b_exp))
            }
            Tree::MetaDataBlock(b_exp) => format!("---\n{}---\n\n", self.render(*b_exp)),
            Tree::MetaDataItem(key, value) => format!("{key}: {value}\n"),
            Tree::Image(b1, b2, b3) => format!(
                "![{}|{}]({})",
                self.render(*b1),
                self.render(*b3),
                self.render(*b2)
            ),
            Tree::Color(b_exp) => format!("\\{{{}}}", self.render(*b_exp)),
            Tree::ImageSizeSpec(b1, b2) => format!("{}x{}", self.render(*b1), self.render(*b2)),
            Tree::VSpace() => String::new(),
            Tree::DropCap(_, _) => todo!(),
            Tree::DocRef(_, _) => todo!(),
            Tree::EmDash => "\u{2014}".to_owned(),
            Tree::EnDash => "\u{2013}".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::*;
    use crate::Render;

    fn render(exp: Tree) -> String {
        Renderer::new().render(exp)
    }

    // --- Primitive nodes ---

    #[test]
    fn literal() {
        assert_eq!(render(lit("hello")), "hello");
    }

    #[test]
    fn escape_lit_renders_as_is() {
        assert_eq!(render(crate::syntax::escape_lit("&")), "&");
    }

    #[test]
    fn preformatted_literal() {
        assert_eq!(render(prelit("code")), "code");
    }

    #[test]
    fn empty_node() {
        assert_eq!(render(empty()), "");
    }

    #[test]
    fn line_break() {
        assert_eq!(render(Tree::LineBreak()), "\n");
    }

    #[test]
    fn paragraph() {
        assert_eq!(render(Tree::Paragraph()), "\n");
    }

    #[test]
    fn vspace() {
        assert_eq!(render(Tree::VSpace()), "");
    }

    #[test]
    fn em_dash() {
        assert_eq!(render(Tree::EmDash), "\u{2014}");
    }

    #[test]
    fn en_dash() {
        assert_eq!(render(Tree::EnDash), "\u{2013}");
    }

    // --- Inline formatting ---

    #[test]
    fn bold_node() {
        assert_eq!(render(Tree::Bold(Box::new(lit("text")))), "*text*");
    }

    #[test]
    fn bold_starting_with_newline_moves_star_to_newline() {
        // If bold content starts with \n, the opening * is moved to the newline
        let exp = Tree::Bold(Box::new(Tree::LineBreak().cat(lit("text"))));
        assert_eq!(render(exp), "\n*text*");
    }

    #[test]
    fn italic_node() {
        assert_eq!(render(Tree::Italic(Box::new(lit("text")))), "_text_");
    }

    #[test]
    fn bold_italic_node() {
        assert_eq!(
            render(Tree::BoldItalic(Box::new(lit("text")))),
            "_**text**_"
        );
    }

    #[test]
    fn small_caps_node() {
        assert_eq!(render(Tree::SmallCaps(Box::new(lit("sc")))), "{sc}");
    }

    #[test]
    fn inline_code_node() {
        assert_eq!(render(Tree::InlineCode(Box::new(lit("x")))), "`x`");
    }

    #[test]
    fn quote_node() {
        assert_eq!(render(Tree::Quote(Box::new(lit("q")))), "\"q\"");
    }

    // --- Block-level nodes ---

    #[test]
    fn heading_level_1() {
        assert_eq!(render(heading(lit("Title"), 0, "")), "# Title");
    }

    #[test]
    fn heading_level_2() {
        assert_eq!(render(heading(lit("Sub"), 1, "")), "## Sub");
    }

    #[test]
    fn code_block_plain() {
        assert_eq!(
            render(Tree::CodeBlock(Box::new(empty()), Box::new(prelit("code\n")))),
            "```\ncode\n```"
        );
    }

    #[test]
    fn code_block_with_type() {
        assert_eq!(
            render(Tree::CodeBlock(
                Box::new(lit("rust")),
                Box::new(prelit("fn f() {}\n"))
            )),
            "```rust\nfn f() {}\n```"
        );
    }

    #[test]
    fn footnote_node() {
        assert_eq!(render(Tree::Footnote(Box::new(lit("note")))), "^(note)");
    }

    #[test]
    fn right_sidenote_node() {
        assert_eq!(
            render(Tree::RightSidenote(Box::new(lit("side")))),
            ">(side)"
        );
    }

    #[test]
    fn chapter_mark_node() {
        assert_eq!(render(Tree::ChapterMark(Box::new(lit("1")))), ">>(1)");
    }

    #[test]
    fn hyperref_node() {
        assert_eq!(
            render(hyperref(lit("text"), lit("http://example.com"))),
            "[text](http://example.com)"
        );
    }

    #[test]
    fn image_node() {
        assert_eq!(
            render(image(
                lit("alt"),
                lit("img.png"),
                image_size(lit("100"), lit("100"))
            )),
            "![alt|100x100](img.png)"
        );
    }

    #[test]
    fn color_node() {
        assert_eq!(render(color(lit("red"))), "\\{red}");
    }

    #[test]
    fn meta_data_block_node() {
        assert_eq!(
            render(meta_data_block(meta_data_item(
                "title".to_string(),
                "My Doc".to_string()
            ))),
            "---\ntitle: My Doc\n---\n\n"
        );
    }

    #[test]
    fn meta_data_item_node() {
        assert_eq!(
            render(meta_data_item("author".to_string(), "Alice".to_string())),
            "author: Alice\n"
        );
    }

    #[test]
    fn list_item_level_0() {
        assert_eq!(render(Tree::ListItem(Box::new(lit("item")), 0)), "item");
    }

    #[test]
    fn list_item_level_1_indented() {
        assert_eq!(render(Tree::ListItem(Box::new(lit("item")), 1)), "  item");
    }

    #[test]
    fn list_renders_inner_content() {
        assert_eq!(render(Tree::List(Box::new(lit("content")), 0)), "content");
    }

    // --- Cat and Document ---

    #[test]
    fn cat_concatenates() {
        assert_eq!(render(lit("a").cat(lit("b"))), "ab");
    }

    #[test]
    fn document_renders_body() {
        assert_eq!(
            render(Tree::Document(
                crate::syntax::DocType::DEFAULT,
                Box::new(lit("body"))
            )),
            "body"
        );
    }

    // --- Word wrap ---

    #[test]
    fn long_line_is_wrapped() {
        let long_word = "word ".repeat(20); // 100 chars, well over wrap column 68
        let output = render(lit(long_word.trim()));
        assert!(output.contains('\n'), "expected a newline from word-wrap");
    }

    #[test]
    fn short_line_is_not_wrapped() {
        let output = render(lit("short line"));
        assert!(!output.contains('\n'));
    }
}
