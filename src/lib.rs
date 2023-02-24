pub mod renderer;
pub mod watch;

use std::{panic, str};

pub fn transform<T: renderer::Renderer>(t: T, input: &str) -> String {
    let result = Parser::parse(input);
    render(t, result)
}

fn render<T: renderer::Renderer>(t: T, exp: Exp) -> String {
    t.render(exp)
}

/// Expressions are the building blocks of an abstract syntax tree
#[derive(Debug)]
pub enum Exp {
    Literal(String),
    Heading(Box<Exp>, u8),
    Bold(Box<Exp>),
    Italic(Box<Exp>),
    Teletype(Box<Exp>),
    Quote(Box<Exp>),
    Footnote(Box<Exp>),
    HyperRef(Box<Exp>, Box<Exp>),
    EscapeLit(String),
    // this enables composition, forming the tree
    Cat(Box<Exp>, Box<Exp>),
    // this is a neutral element, yielding no ouput
    Empty(),
}


impl Exp {
    /// constructs new Exp of self and expr
    fn cat(self, expr: Exp) -> Exp {
        Exp::Cat(Box::new(self), Box::new(expr))
    }
}

// TODO all these to_string invocation incur a copy!
fn lit(s: &str) -> Exp {
    Exp::Literal(s.to_string())
}
fn escape_lit(s: &str) -> Exp {
    Exp::EscapeLit(s.to_string())
}
fn heading(exp: Exp, lvl: u8) -> Exp {
    Exp::Heading(Box::new(exp), lvl)
}
fn footnote(exp: Exp) -> Exp {
    Exp::Footnote(Box::new(exp))
}
fn hyperref(exp1: Exp, exp2: Exp) -> Exp {
    Exp::HyperRef(Box::new(exp1), Box::new(exp2))
}

/// holds parsing state
#[derive(Debug)]
struct Parser<'a> {
    /// the input string as a byte slice
    input: &'a [u8],
    /// the lnegth of the input byte slice
    input_len: usize,
    /// the current position of parsing
    i: usize,
    /// the character at the current parsing position
    char: u8,
}

impl Parser<'_> {
    fn new(input: &str) -> Parser {
        let input_byte_slice = input.as_bytes();
        Parser {
            input: input_byte_slice,
            input_len: input_byte_slice.len(),
            i: 0,
            char: input_byte_slice[0],
        }
    }

    fn parse(input: &str) -> Exp {
        let mut parser = Parser::new(input);
        // passing "" as bytes parses until the end of file
        return parser.parse_until("".as_bytes());
    }
    
    fn advance(&mut self) {
        self.i += 1;
        if !self.at_end() {
            self.char = self.input[self.i];
        }
    }

    fn at_end(&self) -> bool {
        self.i >= self.input_len
    }

    fn consume(&mut self, char: u8) {
        if self.at_end() {
            panic!("index {} out of bounds {} ", self.i, self.input_len);
        }
        if self.char != char {
            panic!(
                "expected char '{}' at index {}, but found '{}'",
                char as char, self.i, self.char as char
            );
        }
        self.advance();
    }

    fn parse_symmetric_quoted(&mut self) -> Box<Exp> {
        let break_char = self.char;
        self.consume(break_char); // opening quote
        let exp = self.parse_until(&[break_char]); // body
        self.consume(break_char); // ending quote
        Box::new(exp)
    }

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
        let literal = self.parse_literal("\n".as_bytes());
        let result = heading(literal, level);
        if self.at_end() {
            return result;
        }
        match self.char {
            b'\n' => result,
            _ => panic!("expected \\n at {}", self.i),
        }
    }

    fn parse_footnote(&mut self) -> Exp {
        self.consume(b'^');
        match self.char {
            b'(' => footnote(self.parse_quoted(b')')),
            _ => lit("^"),
        }
    }

    fn parse_hyperlink(&mut self) -> Exp {
        self.consume(b'[');
        let exp_link_text = self.parse_until("]".as_bytes());
        self.consume(b']');
        if self.char == b'(' {
            self.consume(b'(');
            let exp_url = self.parse_until(")".as_bytes());
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

    fn parse_code(&mut self, ) -> Box<Exp> {
        self.consume(b'`'); // opening quote
        let mut exp = Exp::Empty();
        if self.char == b'.' {
            self.consume(b'.');
            exp = escape_lit(".")
        }
        let exp = exp.cat(self.parse_literal("`".as_bytes()));
        self.consume(b'`'); // closing quote
        Box::new(exp)
    }

    fn parse_until(&mut self, break_chars: &[u8]) -> Exp {
        let mut expression = Exp::Empty(); // we start with "nothing", as rust has no null values
        while !self.at_end() && !break_chars.contains(&self.char) {
            let expr = match self.char {
                b'#' => self.parse_heading(),
                b'*' => Exp::Bold(self.parse_symmetric_quoted()),
                b'_' => Exp::Italic(self.parse_symmetric_quoted()),
                b'`' => Exp::Teletype(self.parse_code()),
                b'"' => Exp::Quote(self.parse_symmetric_quoted()),
                b'^' => self.parse_footnote(),
                b'&' => {
                    self.consume(self.char);
                    escape_lit("&")
                },
                b'.' => {
                    self.consume(self.char);
                    escape_lit(".")
                }
                b'[' => self.parse_hyperlink(),
                _ => self.parse_literal(
                    format!("_*#\"^`&{}", str::from_utf8(break_chars).unwrap()).as_bytes(),
                ),
            };
            expression = expression.cat(expr);
        }
        expression
    }
}
