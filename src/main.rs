use std::fmt::Display;
use std::thread::current;
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

fn consume(input: &[u8], at_index: usize, char: u8) -> usize {
    if at_index >= input.len() {
        panic!("index {} out of bounds {} ", at_index, input.len());
    }
    if input[at_index] != char {
        panic!("expected char '{}' at index {}, but found '{}'", char as char, at_index, input[at_index] as char);
    }
    at_index + 1
}

// TODO merge this function into the parse function
fn parse_literal(input: &[u8], start: usize, break_chars: &[u8]) -> (Exp, usize) {
    let mut current: usize = start;
    while current < input.len() {
        let current_char = input[current];
        if break_chars.contains(&current_char) {
            break;
        }
        current += 1;
    }
    (
        Exp::Literal(str::from_utf8(&input[start..current]).unwrap().to_string()),
        current,
    )
}

fn parse_italic(input: &[u8], current: usize) -> (Exp, usize) {
    let current = consume(input, current, b'_');
    let (literal, current) = parse(input, current, "_".as_bytes());
    let current = consume(input, current, b'_');
    (Exp::Italic(Box::new(literal)), current)
}

fn parse_bold(input: &[u8], current: usize) -> (Exp, usize) {
    let current = consume(input, current, b'*');
    let (literal, current) = parse(input, current, "*".as_bytes());
    let current = consume(input, current, b'*');
    (Exp::Bold(Box::new(literal)), current)
}

fn parse_quote(input: &[u8], current: usize) -> (Exp, usize) {
    let current = consume(input, current, b'"');
    let (literal, current) = parse(input, current, "\"".as_bytes());
    let current = consume(input, current, b'"');
    (Exp::Quote(Box::new(literal)), current)
}

fn parse_heading_level(input: &[u8], start: usize, level: u8) -> (usize, u8) {
    match input[start] {
        b'#' => parse_heading_level(input, start + 1, level + 1),
        b' ' => (start + 1, level),
        _ => (start, level),
    }
}

fn parse_heading(input: &[u8], start: usize) -> (Exp, usize) {
    let start = consume(input, start, b'#');
    let (start, level) = parse_heading_level(input, start, 0);
    let (literal, current) = parse_literal(input, start, "\n".as_bytes());
    let result = (Exp::Heading(Box::new(literal), level), current);
    if current == input.len(){
        return result;        
    }
    match input[current] {
        b'\n' => result,
        _ => panic!("expected \\n at {}", current),
    }
}

fn parse(input: &[u8], mut current: usize, break_chars: &[u8]) -> (Exp, usize) {
    let mut expression = Exp::Empty(); // we start with "nothing", as rust has no null values
    while current < input.len() {
        let current_char = input[current];
        if break_chars.contains(&current_char){
            break;
        }
        let (expr, cur) = match current_char {
            b'#' => parse_heading(input, current),
            b'*' => parse_bold(input, current),
            b'_' => parse_italic(input, current),
            b'"' => parse_quote(input, current),
            _ => parse_literal(input, current, "_*#\"".as_bytes()),
        };
        expression = Exp::Cat(Box::new(expression), Box::new(expr));
        current = cur;
    }
    (expression, current)
}

fn transform(input: &str) -> String {
    return parse(input.as_bytes(), 0, "".as_bytes()).0.to_string();
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
