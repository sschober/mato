use std::fmt::Display;
use std::{env, panic, str};

// Expressions are the building blocks of the abstract syntax tree
#[derive(Debug)]
enum Exp {
    Literal(String),
    Heading(Box<Exp>, u8),
    Bold(Box<Exp>),
    Italic(Box<Exp>),
    Quote(Box<Exp>),
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
            Exp::Heading(b_exp, level) => {
                let section = match level {
                    2 => "subsubsection",
                    1 => "subsection",
                    _ => "section",
                };
                write!(f, "\\{}{{{}}}", section, b_exp)
            }
            Exp::Quote(b_exp) => write!(f, "\"`{}\"'", b_exp),
            Exp::Cat(b_exp1, b_exp2) => write!(f, "{}{}", b_exp1, b_exp2),
            Exp::Empty() => write!(f, ""),
        }
    }
}

// holds parsing state
struct Parser<'a> {
    // the input string as a byte slice
    input: &'a [u8],
    // the lnegth of the input byte slice
    input_len: usize,
    // the current position of parsing
    i: usize,
    // the character at the current parsing position
    char: u8
}

impl Parser<'_> {
    
    fn advance(&mut self) {
        self.i +=1;
        if !self.at_end(){
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

    fn parse_literal(&mut self, break_chars: &[u8]) -> Exp {
        let start = self.i;
        while !self.at_end() {
            if break_chars.contains(&self.char) {
                break;
            }
            self.advance();
        }
        Exp::Literal(
            str::from_utf8(&self.input[start..self.i])
                .unwrap()
                .to_string(),
        )
    }

    fn parse_quoted(&mut self) -> Exp {
        let break_char = self.char;
        self.consume(break_char); // opening quote
        let exp = self.parse(&[break_char]); // body
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
        let result = Exp::Heading(Box::new(literal), level);
        if self.at_end() {
            return result;
        }
        match self.char {
            b'\n' => result,
            _ => panic!("expected \\n at {}", self.i),
        }
    }

    fn parse(&mut self, break_chars: &[u8]) -> Exp {
        let mut expression = Exp::Empty(); // we start with "nothing", as rust has no null values
        while !self.at_end() {
            if break_chars.contains(&self.char) {
                break;
            }
            let expr = match self.char {
                b'#' => self.parse_heading(),
                b'*' => Exp::Bold(Box::new(self.parse_quoted())),
                b'_' => Exp::Italic(Box::new(self.parse_quoted())),
                b'"' => Exp::Quote(Box::new(self.parse_quoted())),
                _ => self.parse_literal("_*#\"".as_bytes()),
            };
            expression = Exp::Cat(Box::new(expression), Box::new(expr));
        }
        expression
    }
}

fn transform(input: &str) -> String {
    let input_byte_slice = input.as_bytes();
    let mut parser = Parser {
        input: input_byte_slice,
        input_len: input_byte_slice.len(),
        i: 0,
        char: input_byte_slice[0]
    };
    return parser.parse("".as_bytes()).to_string();
}

fn main() {
    for file in env::args().skip(1) {
        let input = std::fs::read_to_string(file).unwrap();
        let result = transform(input.as_str());
        println!("{}", result);
    }
}

mod tests {

    #[test]
    fn literal() {
        assert_eq!(super::transform("hallo"), "hallo");
    }
    #[test]
    fn italic() {
        assert_eq!(super::transform("_hallo_"), "\\textit{hallo}");
    }
    #[test]
    fn bold() {
        assert_eq!(super::transform("*hallo*"), "\\textbf{hallo}");
    }
    #[test]
    fn heading() {
        assert_eq!(super::transform("# heading\n"), "\\section{heading}\n");
    }
    #[test]
    fn heading_without_newline() {
        assert_eq!(super::transform("# 1"), "\\section{1}");
    }
    #[test]
    fn quote() {
        assert_eq!(super::transform("\"input\""), "\"`input\"'");
    }
    #[test]
    fn bold_and_italic() {
        assert_eq!(super::transform("*_text_*"), "\\textbf{\\textit{text}}");
    }
    #[test]
    fn bold_and_italic_but_with_outer_chars() {
        assert_eq!(
            super::transform("*fett _kursiv_ wieder fett*"),
            "\\textbf{fett \\textit{kursiv} wieder fett}"
        );
    }
}
