use std::fmt::Display;
use std::{panic, str};

// Expressions are the building blocks of the abstract syntax tree
#[derive(Debug)]
enum Expression {
    Literal(String),
    Heading(Box<Expression>),
    Bold(Box<Expression>),
    Italic(Box<Expression>),
    // this enables composition, forming the tree
    Concat(Box<Expression>, Box<Expression>),
    // this is a neutral element, yielding no ouput
    Empty(),
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Literal(s) => write!(f, "{}", s),
            Expression::Bold(b_exp) => write!(f, "\\textbf{{{}}}", b_exp),
            Expression::Italic(b_exp) => write!(f, "\\textit{{{}}}", b_exp),
            Expression::Heading(b_exp) => write!(f, "\\section{{{}}}", b_exp),
            Expression::Concat(b_exp1, b_exp2) => write!(f, "{}{}", b_exp1, b_exp2),
            Expression::Empty() => write!(f, ""),
        }
    }
}

fn parse_literal(input: &[u8], start: usize, break_chars: &[u8]) -> (Expression, usize) {
    let mut current: usize = start;
    while current < input.len() {
        let current_char = input[current];
        if break_chars.contains(&current_char) {
            break;
        }
        current += 1;
    }
    (
        Expression::Literal(str::from_utf8(&input[start..current]).unwrap().to_string()),
        current,
    )
}

fn parse_italic(input: &[u8], start: usize) -> (Expression, usize) {
    let (literal, current) = parse_literal(input, start, "_*".as_bytes());
    match input[current] {
        b'_' => (Expression::Italic(Box::new(literal)), current + 1), // the +1 consumes the '_'
        // having no arm for '*' means we cannot nest a '*' in a "_", like so '_*kursiv und fett*_'
        _ => panic!("expected _ at {}", current),
    }
}

fn parse_bold(input: &[u8], start: usize) -> (Expression, usize) {
    let (literal, current) = parse_literal(input, start, "_*".as_bytes());
    match input[current] {
        b'*' => (Expression::Bold(Box::new(literal)), current + 1), // the +1 consumes the '*'
        // no nesting
        _ => panic!("expected * at {}", current),
    }
}

fn parse_heading(input: &[u8], start: usize) -> (Expression, usize) {
    let (literal, current) = parse_literal(input, start, "\n".as_bytes());
    match input[current] {
        b'\n' => (Expression::Heading(Box::new(literal)), current),
        _ => panic!("expected \\n at {}", current),
    }
}

fn parse(input: &[u8], start: usize) -> Expression {
    let mut expression = Expression::Empty(); // we start with "nothing", as rust has no null values
    let mut current: usize = start;
    while current < input.len() {
        let current_char = input[current];
        let (expr, next_pos) = match current_char {
            b'#' => parse_heading(input, current + 1),
            b'*' => parse_bold(input, current + 1),
            b'_' => parse_italic(input, current + 1),
            _ => parse_literal(input, current, "_*".as_bytes()),
        };
        expression = Expression::Concat(Box::new(expression), Box::new(expr));
        current = next_pos;
    }
    expression
}

fn main() {
    let literal = "# titel\nhallo _kursive_ welt *i*\nhallo nächste zeile\n__";
    let input = literal.as_bytes();
    let result = parse(input, 0);
    println!("{:?}", result);
    println!("{}", result);
}

mod tests {
    use super::*;

    #[test]
    fn literal() {
        assert_eq!(
            Expression::Literal("hallo".to_string()).to_string(),
            "hallo"
        );
    }
    #[test]
    fn italic() {
        assert_eq!(
            Expression::Italic(Box::new(Expression::Literal("hallo".to_string()))).to_string(),
            "\\textit{hallo}"
        );
    }
    #[test]
    fn bold() {
        assert_eq!(
            Expression::Bold(Box::new(Expression::Literal("hallo".to_string()))).to_string(),
            "\\textbf{hallo}"
        );
    }
    #[test]
    fn concat() {
        assert_eq!(
            Expression::Concat(
                Box::new(Expression::Literal("hallo".to_string())),
                Box::new(Expression::Literal(" welt".to_string()))
            )
            .to_string(),
            "hallo welt"
        );
    }

    #[test]
    fn nested_concat() {
        assert_eq!(
            Expression::Concat(
                Box::new(Expression::Literal("hallo".to_string())),
                Box::new(Expression::Concat(
                    Box::new(Expression::Literal(" kursive".to_string())),
                    Box::new(Expression::Literal(" welt".to_string()))
                ))
            )
            .to_string(),
            "hallo kursive welt"
        );
    }

    #[test]
    fn nested_concat_and_italic() {
        assert_eq!(
            Expression::Concat(
                Box::new(Expression::Literal("hallo".to_string())),
                Box::new(Expression::Concat(
                    Box::new(Expression::Italic(Box::new(Expression::Literal(
                        " kursive".to_string()
                    )))),
                    Box::new(Expression::Literal(" welt".to_string()))
                ))
            )
            .to_string(),
            "hallo\\textit{ kursive} welt"
        );
    }
}
