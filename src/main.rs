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
struct Parser<'a> {
    input: &'a[u8],
    input_len : usize,
    i: usize
}

impl Parser<'_> {

    fn consume(&mut self, char: u8) {
        if self.i >= self.input_len {
            panic!("index {} out of bounds {} ", self.i, self.input_len);
        }
        if self.input[self.i] != char {
            panic!("expected char '{}' at index {}, but found '{}'", char as char, self.i, self.input[self.i] as char);
        }
        self.i += 1;
    }

    fn parse_literal(&mut self, break_chars: &[u8]) -> Exp {
        let start = self.i;
        while self.i < self.input_len {
            if break_chars.contains(&self.input[self.i]) {
                break;
            }
            self.i += 1;
        }
        Exp::Literal(str::from_utf8(&self.input[start..self.i]).unwrap().to_string())
    }

    fn parse_italic(&mut self) -> Exp {
        self.consume(b'_');
        let literal= self.parse("_".as_bytes());
        self.consume(b'_');
        Exp::Italic(Box::new(literal))
    }

    fn parse_bold(&mut self) -> Exp {
        self.consume(b'*');
        let literal = self.parse("*".as_bytes());
        self.consume(b'*');
        Exp::Bold(Box::new(literal))
    }

    fn parse_quote(&mut self) -> Exp {
        self.consume(b'"');
        let literal = self.parse("\"".as_bytes());
        self.consume(b'"');
        Exp::Quote(Box::new(literal))
    }

    fn parse_heading_level(&mut self, level: u8) -> u8 {
        match self.input[self.i] {
            b'#' => {
                self.i += 1;
                self.parse_heading_level(level + 1)
            },
            b' ' => {
                self.i += 1;
                level
            },
            _ => level,
        }
    }

    fn parse_heading(&mut self) -> Exp {
        self.consume(b'#');
        let level = self.parse_heading_level(0);
        let literal = self.parse_literal("\n".as_bytes());
        let result = Exp::Heading(Box::new(literal), level);
        if self.i == self.input_len {
            return result;        
        }
        match self.input[self.i] {
            b'\n' => result,
            _ => panic!("expected \\n at {}", self.i),
        }
    }

    fn parse(&mut self, break_chars: &[u8]) -> Exp {
        let mut expression = Exp::Empty(); // we start with "nothing", as rust has no null values
        while self.i < self.input_len {
            let current_char = self.input[self.i];
            if break_chars.contains(&current_char){
                break;
            }
            let expr = match current_char {
                b'#' => self.parse_heading(),
                b'*' => self.parse_bold(),
                b'_' => self.parse_italic(),
                b'"' => self.parse_quote(),
                _ => self.parse_literal("_*#\"".as_bytes()),
            };
            expression = Exp::Cat(Box::new(expression), Box::new(expr));
        }
        expression
    }
    
}

fn transform(input: &str) -> String {
    let mut parser = Parser {input : input.as_bytes(), input_len : input.len(), i : 0};
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
    fn heading(){
        assert_eq!(super::transform("# heading\n"), "\\section{heading}\n");
    }
    #[test]
    fn heading_without_newline(){
        assert_eq!(super::transform("# 1"), "\\section{1}");
    }
    #[test]
    fn quote(){
        assert_eq!(super::transform("\"input\""), "\"`input\"'");
    }
    #[test]
    fn bold_and_italic(){
        assert_eq!(super::transform("*_text_*"), "\\textbf{\\textit{text}}");
    }
    #[test]
    fn bold_and_italic_but_with_outer_chars(){
        assert_eq!(super::transform("*fett _kursiv_ wieder fett*"), "\\textbf{fett \\textit{kursiv} wieder fett}");
    }
}
