use std::fmt::Display;
use std::{panic, str};

/// Expressions are the building blocks of an abstract syntax tree
#[derive(Debug)]
enum Exp {
    Literal(String),
    Heading(Box<Exp>, u8),
    Bold(Box<Exp>),
    Italic(Box<Exp>),
    Teletype(Box<Exp>),
    Quote(Box<Exp>),
    Footnote(Box<Exp>),
    HyperRef(Box<Exp>, Box<Exp>),
    // this enables composition, forming the tree
    Cat(Box<Exp>, Box<Exp>),
    // this is a neutral element, yielding no ouput
    Empty(),
}

impl Display for Exp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Exp::Literal(s) => write!(f, "{}", s),
            Exp::Bold(b_exp) => write!(f, "\\textbf{{{}}}", b_exp),
            Exp::Italic(b_exp) => write!(f, "\\textit{{{}}}", b_exp),
            Exp::Teletype(b_exp) => write!(f, "\\texttt{{{}}}", b_exp),
            Exp::Heading(b_exp, level) => {
                let section = match level {
                    2 => "subsubsection",
                    1 => "subsection",
                    _ => "section",
                };
                write!(f, "\\{}{{{}}}", section, b_exp)
            }
            Exp::Quote(b_exp) => write!(f, "\"`{}\"'", b_exp),
            Exp::Footnote(b_exp) => write!(f, "~\\footnote{{{}}}", b_exp),
            Exp::HyperRef(b_exp1, b_exp2) => write!(f, "\\href{{{}}}{{{}}}", b_exp2, b_exp1),
            Exp::Cat(b_exp1, b_exp2) => write!(f, "{}{}", b_exp1, b_exp2),
            Exp::Empty() => write!(f, ""),
        }
    }
}

impl Exp {
    /// constructs new Exp of self and expr
    fn cat(self, expr: Exp) -> Exp {
        Exp::Cat(Box::new(self), Box::new(expr))
    }
}

fn lit(s: &str) -> Exp {
    Exp::Literal(s.to_string())
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
pub struct Parser<'a> {
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

    pub fn transform(input: &str) -> String {
        let mut parser = Parser::new(input);
        // passing "" as bytes parses until the end of file
        return parser.parse_until("".as_bytes()).to_string();
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

    fn parse_until(&mut self, break_chars: &[u8]) -> Exp {
        let mut expression = Exp::Empty(); // we start with "nothing", as rust has no null values
        while !self.at_end() && !break_chars.contains(&self.char) {
            let expr = match self.char {
                b'#' => self.parse_heading(),
                b'*' => Exp::Bold(self.parse_symmetric_quoted()),
                b'_' => Exp::Italic(self.parse_symmetric_quoted()),
                b'`' => Exp::Teletype(self.parse_symmetric_quoted()),
                b'"' => Exp::Quote(self.parse_symmetric_quoted()),
                b'^' => self.parse_footnote(),
                b'&' => {
                    self.consume(self.char);
                    lit("\\&")
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
