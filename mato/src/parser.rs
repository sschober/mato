use crate::syntax::{escape_lit, footnote, heading, hyperref, lit, Exp};
use std::str;

/// holds parsing state
#[derive(Debug)]
pub struct Parser<'a> {
    /// the input string as a byte slice
    input: &'a [u8],
    /// the length of the input byte slice
    input_len: usize,
    /// the current position of parsing
    i: usize,
    /// the character at the current parsing position
    char: u8,
}

impl Parser<'_> {
    const fn new(input: &str) -> Parser<'_> {
        let input_byte_slice = input.as_bytes();
        Parser {
            input: input_byte_slice,
            input_len: input_byte_slice.len(),
            i: 0,
            char: input_byte_slice[0],
        }
    }

    #[must_use]
    pub fn parse(input: &str) -> Exp {
        let mut parser = Parser::new(input);
        // passing "" as bytes parses until the end of file
        parser.parse_until(b"")
    }

    /// increases index and updates current char
    fn advance(&mut self) {
        self.i += 1;
        if !self.at_end() {
            self.char = self.input[self.i];
        }
    }

    /// true, if current index is equal to or greater than the
    /// input string length
    const fn at_end(&self) -> bool {
        self.i >= self.input_len
    }

    const fn peek(&self, n: usize, char: u8) -> bool {
        if self.i + n >= self.input_len {
            false
        } else {
            char == self.input[self.i + n]
        }
    }

    /// eat up a given character, or panic if that is not found at
    /// the current position or we are already at the end of the
    /// input string
    fn consume(&mut self, char: u8) {
        assert!(
            !self.at_end(),
            "consume({}): index {} out of bounds {} ",
            char as char,
            self.i,
            self.input_len
        );
        assert!(
            self.char == char,
            "expected char '{}' at index {}, but found '{}'",
            char as char,
            self.i,
            self.char as char
        );
        self.advance();
    }

    /// parse a symmetrically quoted sub string, like
    /// something enclosed in a " pair
    fn parse_symmetric_quoted(&mut self) -> Exp {
        let break_char = self.char;
        self.consume(break_char); // opening quote
        let exp = self.parse_until(&[break_char]); // body
        self.consume(break_char); // ending quote
        exp
    }

    /// parse an asymmetrically quoted substring, like
    /// something enclosed in a pair of parentheses, ( and ).
    fn parse_quoted(&mut self, break_char: u8) -> Exp {
        self.consume(self.char); // opening quote
        let exp = self.parse_until(&[break_char]); // body
        self.consume(break_char); // ending quote
        exp
    }

    fn parse_heading_level(&mut self, level: u8) -> u8 {
        match self.char {
            b'#' => {
                self.advance();
                self.parse_heading_level(level + 1)
            }
            b' ' => {
                self.advance();
                level
            }
            _ => level,
        }
    }

    fn parse_heading(&mut self) -> Exp {
        self.consume(b'#');
        let level = self.parse_heading_level(0);
        let literal = self.parse_literal(b"\n");
        let result = heading(literal, level);
        if self.at_end() {
            return result;
        }
        if self.peek(2, b'#') && level != 2 {
            // if this heading is followed by another heading, we slurp away the newline
            // so that there is not too much vertical white space in between them
            self.consume(b'\n');
        }
        result
    }

    fn parse_footnote(&mut self) -> Exp {
        self.consume(b'^');
        match self.char {
            b'(' => footnote(self.parse_quoted(b')')),
            _ => lit("^"),
        }
    }

    fn parse_right_sidenote(&mut self) -> Exp {
        self.consume(b'>');
        let is_chapter_mark = self.char == b'>';
        if is_chapter_mark {
            self.consume(b'>');
        }
        match self.char {
            b'(' => {
                let box_exp = Box::new(self.parse_quoted(b')'));
                let exp = if is_chapter_mark {
                    self.consume(b'\n');
                    Exp::ChapterMark(box_exp)
                } else {
                    Exp::RightSidenote(box_exp)
                };
                if self.char == b' ' {
                    self.consume(b' ');
                }
                exp
            }
            _ => {
                if is_chapter_mark {
                    lit(">>")
                } else {
                    lit(">")
                }
            }
        }
    }

    fn parse_hyperlink(&mut self) -> Exp {
        self.consume(b'[');
        let exp_link_text = self.parse_until(b"]");
        self.consume(b']');
        if self.char == b'(' {
            self.consume(b'(');
            let exp_url = self.parse_literal(b")");
            self.consume(b')');
            hyperref(exp_link_text, exp_url)
        } else {
            lit("[").cat(exp_link_text).cat(lit("]"))
        }
    }

    fn parse_literal(&mut self, break_chars: &[u8]) -> Exp {
        let start = self.i;
        while !self.at_end() && !break_chars.contains(&self.char) {
            self.advance();
        }
        lit(str::from_utf8(&self.input[start..self.i]).unwrap())
    }

    fn parse_code(&mut self) -> Exp {
        self.consume(b'`'); // opening quote
        let mut is_code_block = false;
        // here, we need to peek 1 and 2 characters ahead to see if
        // they are also back ticks, and if so parse a code block
        // instead of an inline code snippet.
        if self.peek(0, b'`') && self.peek(1, b'`') {
            // parse code block
            is_code_block = true;
            println!("code block detected!");
            self.consume(b'`');
            self.consume(b'`');
            self.consume(b'\n');
        }

        // this is an ugly groff necessity: if our code snippet
        // begins with a dot, we need to escape it
        let exp = if self.char == b'.' {
            self.consume(b'.');
            escape_lit(".")
        } else {
            Exp::Empty()
        };

        let exp = exp.cat(self.parse_literal(b"`"));
        self.consume(b'`'); // closing quote
        if is_code_block {
            self.consume(b'`'); // closing quote
            self.consume(b'`'); // closing quote
            if ! self.at_end() {
                // comsuming the newline is optional, as the code block 
                // might be the last element in the file and might not
                // end with a newline (been there, done that)
                self.consume(b'\n'); // extra newline
            }
            Exp::CodeBlock(Box::new(exp))
        } else {
            Exp::InlineCode(Box::new(exp))
        }
    }

    fn parse_until(&mut self, break_chars: &[u8]) -> Exp {
        let mut expression = Exp::Empty(); // we start with
                                           // "nothing", as rust has
                                           // no null values
        while !self.at_end() && !break_chars.contains(&self.char) {
            let expr = match self.char {
                b'#' => self.parse_heading(),
                b'*' => Exp::Bold(Box::new(self.parse_symmetric_quoted())),
                b'_' => Exp::Italic(Box::new(self.parse_symmetric_quoted())),
                b'`' => self.parse_code(),
                b'"' => Exp::Quote(Box::new(self.parse_symmetric_quoted())),
                b'^' => self.parse_footnote(),
                b'&' => {
                    self.consume(self.char);
                    escape_lit("&")
                }
                b'.' => {
                    self.consume(self.char);
                    escape_lit(".")
                }
                b'[' => self.parse_hyperlink(),
                b'\n' => {
                    // if the blank line is followed by a heading do not insert a paragraph
                    if self.peek(1, b'\n') && !self.peek(2, b'#') {
                        self.consume(b'\n');
                        Exp::Paragraph()
                    } else {
                        self.consume(b'\n');
                        lit("\n")
                    }
                }
                b'>' => self.parse_right_sidenote(),
                _ => self.parse_literal(
                    format!("_*#\"^`&[{}>\n", str::from_utf8(break_chars).unwrap()).as_bytes(),
                ),
            };
            expression = expression.cat(expr);
        }
        expression
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;
    #[test]
    fn construction() {
        let parser = Parser::new("\"quoted\"");
        assert_eq!(format!("{:?}", parser), "Parser { input: [34, 113, 117, 111, 116, 101, 100, 34], input_len: 8, i: 0, char: 34 }");
    }
    #[test]
    fn expression() {
        let parser = Parser::parse("\"quoted\"");
        assert_eq!(
            format!("{:?}", parser),
            "Cat(Empty, Quote(Cat(Empty, Literal(\"quoted\"))))"
        );
    }
    #[test]
    fn ampersand() {
        let parser = Parser::parse("&");
        assert_eq!(format!("{:?}", parser), "Cat(Empty, EscapeLit(\"&\"))");
    }
    #[test]
    fn dot() {
        let parser = Parser::parse(".");
        assert_eq!(format!("{:?}", parser), "Cat(Empty, EscapeLit(\".\"))");
    }
}
