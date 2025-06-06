use crate::syntax::{
    bold, color, empty, escape_lit, footnote, heading, hyperref, image, image_size, list,
    list_item, lit, meta_data_item, prelit, DocType, Tree,
};
use std::str;

/// holds parsing state
#[derive(Debug)]
pub struct Parser<'a> {
    /// the input string as a byte slice
    input: &'a [u8],
    /// the length of the input byte slice
    input_len: usize,
    /// the current position of parsing
    current_position: usize,
    current_line: usize,
    /// the character at the current parsing position
    current_char: u8,
    doc_type: String,
}

/// indentation unit of lists in spaces
const LIST_INDENT: u8 = 2;

impl Parser<'_> {
    fn new(input: &str) -> Parser<'_> {
        let input_byte_slice = input.as_bytes();
        Parser {
            input: input_byte_slice,
            input_len: input_byte_slice.len(),
            current_position: 0,
            current_char: input_byte_slice[0],
            current_line: 1,
            doc_type: "".to_owned(),
        }
    }

    #[must_use]
    pub fn parse(input: &str) -> Tree {
        if input.is_empty() {
            Tree::Document(DocType::DEFAULT, Box::new(empty()))
        } else {
            let mut parser = Parser::new(input);
            // passing "" as bytes parses until the end of file
            let ast = Box::new(parser.parse_complete_until(b""));
            match parser.doc_type.to_uppercase().as_ref() {
                "SLIDES" => Tree::Document(DocType::SLIDES, ast),
                "CHAPTER" => Tree::Document(DocType::CHAPTER, ast),
                "LETTER" => Tree::Document(DocType::LETTER, ast),
                _ => Tree::Document(DocType::DEFAULT, ast),
            }
        }
    }

    /// increase index, update current char and increase line counter if newline is encountered.
    /// if after advancing the index, we are at the end of the input, we set current char to EOF.
    fn advance(&mut self) {
        if self.current_char == b'\n' {
            self.current_line += 1;
        }
        self.current_position += 1;
        if !self.at_end() {
            self.current_char = self.input[self.current_position];
        } else {
            self.current_char = 4; // EOF
        }
    }

    /// true, if current index is equal to or greater than the
    /// input string length
    const fn at_end(&self) -> bool {
        self.current_position >= self.input_len
    }

    /// compare backwards `n` characters with `char`. return `true`, if the character at that
    /// position is equal to the passed in character.
    fn peek_back(&self, n: usize, char: u8) -> bool {
        let idx: i32 = self.current_position as i32 - n as i32;
        if idx < 0 {
            false
        } else {
            let char_at = self.input[idx as usize];
            char == char_at
        }
    }

    /// compare forward `n` characters with `char`
    const fn peek(&self, n: usize, char: u8) -> bool {
        if self.current_position + n >= self.input_len {
            false
        } else {
            char == self.input[self.current_position + n]
        }
    }

    /// eat up a given character, or panic if that is not found at
    /// the current position or if we are already at the end of the
    /// input string
    fn consume(&mut self, char: u8) {
        assert!(
            !self.at_end(),
            "at line {}: consume({}): index {} out of bounds {}; was expecting '{}' but hit EOF before.",
            self.current_line,
            char as char,
            self.current_position,
            self.input_len,
            char as char
        );
        assert!(
            self.current_char == char,
            "at line {}: expected char '{}' at index {}, but found '{}'",
            self.current_line,
            char as char,
            self.current_position,
            self.current_char as char
        );
        self.advance();
    }

    /// parse a symmetrically quoted sub string, like
    /// something enclosed in a " pair
    fn parse_symmetric_quoted(&mut self) -> Tree {
        let break_char = self.current_char;
        self.consume(break_char); // opening quote
        let exp = self.parse_complete_until(&[break_char]); // body
        self.consume(break_char); // ending quote
        exp
    }

    fn try_bold_or_lit_until(&mut self, break_chars: &[u8]) -> Tree {
        let break_char = self.current_char;
        self.consume(break_char); // opening quote
        let exp = self.parse_format_until(
            format!(
                "{}{}",
                break_char as char,
                str::from_utf8(break_chars).unwrap()
            )
            .as_bytes(),
        ); // body
        if self.current_char == break_char {
            self.consume(break_char); // ending quote
            Tree::Bold(Box::new(exp))
        } else {
            eprintln!("current char: {}, {:?}", self.current_char as char, exp);
            // we consumed the '*', so we prepend it again
            lit("*").cat(exp)
        }
    }

    /// parses something that is asymmetrically quoted, like with '(' and ')'
    /// takes a function pointer of sorts, to parse the inner stuff
    fn parse_quoted_base(
        &mut self,
        break_char: u8,
        func: for<'a, 'b> fn(&'a mut Parser, &'b [u8]) -> Tree,
    ) -> Tree {
        self.consume(self.current_char); // opening quote
        let exp = func(self, &[break_char]); // body
        self.consume(break_char); // ending quote
        exp
    }

    /// parse an asymmetrically quoted substring, like
    /// something enclosed in a pair of parentheses, ( and ).
    fn parse_quoted(&mut self, break_char: u8) -> Tree {
        self.parse_quoted_base(break_char, |a, b| Parser::parse_complete_until(a, b))
    }

    /// parse an asymmetrically quoted substring, like
    /// something enclosed in a pair of parentheses, ( and ).
    fn parse_quoted_literal(&mut self, break_char: u8) -> Tree {
        self.parse_quoted_base(break_char, |a, b| Parser::parse_literal(a, b))
    }

    /// advances over input string until a non-hash character is encountered and returns number of
    /// encountered hash characters.
    fn parse_heading_level(&mut self, level: u8) -> u8 {
        match self.current_char {
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

    /// parse markdown headings of unspecified level:
    ///
    ///     `# heading`
    ///
    /// is a level 1 heading,
    ///
    ///     `## heading`
    ///
    /// is a level 2 heading.
    fn parse_heading(&mut self) -> Tree {
        self.consume(b'#');
        let level = self.parse_heading_level(0);
        let literal = self.parse_literal(b"/\n");
        let mut heading_name = "".to_string();
        if self.current_char == b'/' {
            self.consume(b'/');
            heading_name = self.parse_string_until(b"/");
            self.consume(b'/');
        }
        let result = heading(literal, level, &heading_name);
        if self.at_end() {
            return result;
        }
        if self.peek(2, b'#') && level != 2 {
            // this heading is followed by another heading, so we slurp away the newline
            // so that there is not too much vertical white space in between them
            self.consume(b'\n');
            result
        } else {
            // heading is not followed by another heading, so we insert a VSpace node.
            result.cat(Tree::VSpace())
        }
    }

    /// prase a markdown footnote, of the form:
    ///
    /// `^(some foot note text)`
    ///
    /// if the tilde `^` is not followed by `(`, a literal tilde is returned
    fn parse_footnote(&mut self) -> Tree {
        self.consume(b'^');
        match self.current_char {
            b'(' => footnote(self.parse_quoted(b')')),
            _ => lit("^"),
        }
    }

    fn count_drop_cap_level(&mut self) -> u8 {
        let mut level = 0;
        while self.current_char == b'%' {
            level += 1;
            self.consume(b'%');
        }
        level
    }

    fn parse_drop_cap(&mut self) -> Tree {
        if self.current_position == 0 || self.peek_back(1, b'\n') {
            // we are at the beginning of the file,
            // or at the beginning of a line
            let drop_cap_level = self.count_drop_cap_level();
            let drop_cap_char = self.current_char;
            self.advance();
            // we increase the drop cap level by one, as a drop
            // cap of 1 does not make any sense. so if the user
            // specifies a single '%' we have a character that
            // drops _one_ line and spans _two_.
            Tree::DropCap(drop_cap_char, drop_cap_level + 1)
        } else {
            self.consume(b'%');
            lit("%")
        }
    }

    fn parse_color_spec(&mut self) -> Tree {
        self.consume(b'\\');
        match self.current_char {
            b'{' => color(self.parse_quoted_literal(b'}')),
            _ => lit("\\"),
        }
    }

    fn parse_right_sidenote(&mut self) -> Tree {
        self.consume(b'>');
        let is_chapter_mark = self.current_char == b'>';
        if is_chapter_mark {
            self.consume(b'>');
        }
        match self.current_char {
            b'(' => {
                let box_exp = Box::new(self.parse_quoted(b')'));
                let exp = if is_chapter_mark {
                    self.consume(b'\n');
                    Tree::ChapterMark(box_exp)
                } else {
                    Tree::RightSidenote(box_exp)
                };
                if self.current_char == b' ' {
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

    fn parse_link(&mut self) -> Tree {
        self.consume(b'[');
        let link_text = self.parse_format_until(b"]");
        self.consume(b']');
        if self.current_char == b'(' {
            self.consume(b'(');
            if self.current_char == b'#' {
                // internal link
                self.consume(b'#');
                let target = self.parse_string_until(b")");
                self.consume(b')');
                Tree::DocRef(target, Box::new(link_text))
            } else {
                // hyper link
                let exp_url = self.parse_literal(b")");
                self.consume(b')');
                // if there is a space after the hyperlink, we swollow it
                // to avoid a line break in the PDf after the link
                if self.current_char == b' ' {
                    self.consume(b' ')
                }
                hyperref(link_text, exp_url)
            }
        } else {
            lit("[").cat(link_text).cat(lit("]"))
        }
    }

    fn parse_raw_until(&mut self, break_chars: &[u8]) -> &[u8] {
        let start = self.current_position;
        while !self.at_end() && !break_chars.contains(&self.current_char) {
            self.advance();
        }
        &self.input[start..self.current_position]
    }

    fn parse_string_until(&mut self, break_chars: &[u8]) -> String {
        str::from_utf8(self.parse_raw_until(break_chars))
            .unwrap()
            .to_string()
    }

    fn parse_literal(&mut self, break_chars: &[u8]) -> Tree {
        lit(&self.parse_string_until(break_chars))
    }

    fn parse_preformatted_literal(&mut self, break_chars: &[u8]) -> Tree {
        prelit(&self.parse_string_until(break_chars))
    }

    /// a 'pass through' is a command written in the source markdown language, but to be passed on
    /// or through to the target language, like groff
    fn parse_pass_through(&mut self) -> Tree {
        self.consume(b'/');
        if self.peek(0, b'/') {
            self.consume(b'/');
            self.parse_literal(b"\n")
        } else {
            lit("/")
        }
    }

    fn is_all_space_until(&self, index: u8) -> bool {
        for i in 0..index as usize {
            if !self.peek(i, b' ') {
                return false;
            }
        }
        true
    }

    fn consume_all_space_until(&mut self, index: u8) {
        for _ in 0..index {
            self.consume(b' ');
        }
    }

    fn consume_all_space(&mut self) {
        while !self.at_end() && self.current_char == b' ' {
            self.consume(b' ')
        }
    }

    /// parse a key value pair in a meta data header. key and value are delimited by `:`.
    fn parse_meta_data_item(&mut self) -> Tree {
        //println!("parsing metadata header");
        let key = self.parse_string_until(b":");
        self.consume(b':');
        while self.current_char == b' ' {
            self.advance();
        }
        let value = self.parse_string_until(b"\n");
        if "doctype" == key {
            println!("setting docype {}", value);
            self.doc_type = value;
            empty()
        } else {
            meta_data_item(key.to_string(), value.to_string())
        }
    }

    /// parse a list of key value items in a meta data block
    fn parse_meta_data_items(&mut self) -> Tree {
        let mut items = empty();
        while self.current_char != b'-' && self.current_char != b'\n' {
            items = items.cat(self.parse_meta_data_item());
            self.consume(b'\n')
        }
        items
    }

    /// try to parse a meta data block. such blocks beginn with three `---` on a line, followed by
    /// a key value list of undefined length and end with a `---` line.
    fn parse_mdb_or_list_or_lit(&mut self) -> Tree {
        //println!("parsing meta data block");
        if self.peek(1, b'-') && self.peek(2, b'-') {
            self.consume(b'-');
            self.consume(b'-');
            self.consume(b'-');
            while self.current_char == b' ' || self.current_char == b'\t' {
                self.advance()
            }
            self.consume(b'\n');
            let items = self.parse_meta_data_items();
            if self.current_char == b'-' {
                self.consume(b'-');
                self.consume(b'-');
                self.consume(b'-');
            }
            self.consume(b'\n');
            if self.current_char == b'\n' {
                self.consume(b'\n');
            }
            Tree::MetaDataBlock(Box::new(items))
        } else if self.peek(1, b' ') {
            self.parse_list_or_bold_or_lit(0, b'-')
        } else {
            self.advance();
            lit("-")
        }
    }

    fn parse_list_item(&mut self, level: u8, list_char: u8) -> Tree {
        let mut item = empty();
        self.consume_all_space_until(level * LIST_INDENT);
        self.consume(list_char);
        self.consume(b' ');
        loop {
            item = item.cat(self.parse_complete_until(b"\n"));
            if !self.at_end() {
                self.consume(b'\n');
            }
            if self.is_all_space_until((level * LIST_INDENT) + LIST_INDENT)
                && !self.peek(
                    (level * LIST_INDENT) as usize + LIST_INDENT as usize,
                    list_char,
                )
            {
                self.consume_all_space_until((level * LIST_INDENT) + LIST_INDENT);
                // reappend the newline we swallowed above
                item = item.cat(lit("\n"));
                continue;
            }
            break;
        }
        list_item(item, level)
    }

    fn parse_list_or_bold_or_lit(&mut self, level: u8, list_char: u8) -> Tree {
        // we try to decide if the author wanted to start a bold segment or write a list:
        // ATM we simply check, if c + (l * li) + 1 is a white space
        if self.peek((level * LIST_INDENT) as usize + 1, b' ') {
            // if * is followed by white space
            self.parse_list(level, list_char)
        } else if list_char == b'*' {
            self.parse_bold()
        } else {
            lit(&format!("{}", list_char as char))
        }
    }

    fn parse_list(&mut self, level: u8, list_char: u8) -> Tree {
        let mut iterator = empty();
        loop {
            if self.peek((level * LIST_INDENT) as usize, list_char)
                && self.peek((level * LIST_INDENT) as usize + 1, b' ')
            {
                iterator = iterator.cat(self.parse_list_item(level, list_char));
                continue;
            } else if self.peek(((level + 1) * LIST_INDENT) as usize, list_char)
                && self.peek(((level + 1) * LIST_INDENT) as usize + 1, b' ')
            {
                // nested list
                iterator = iterator.cat(self.parse_list_or_bold_or_lit(level + 1, list_char));
            } else {
                break;
            }
        }
        list(iterator, level)
    }
    fn parse_bold(&mut self) -> Tree {
        if self.peek(1, b'*') {
            self.consume(b'*')
        }
        let res = bold(self.parse_symmetric_quoted());
        if self.current_char == b'*' {
            self.consume(b'*')
        }
        res
    }
    fn parse_code_block(&mut self) -> Tree {
        let mut block_type = empty();
        self.consume(b'`');
        self.consume(b'`');
        self.consume_all_space(); // slurp away aditional white space
        if self.current_char != b'\n' {
            block_type = self.parse_literal(b"\n");
        }
        self.consume(b'\n');
        // groff requires '.'s at the beginning of the line to be escaped, even in code blocks;
        // maybe, we can change the character in code blocks, temporarily
        // TODO escape '.' at line beginnings
        let mut result = self.parse_preformatted_literal(b"`");
        // when parse_preformatted_literal returns, it encountered a ` char we are in a code block
        // and such a block is only ended by three backticks on a line
        while !(self.at_end() || self.peek(1, b'`') && self.peek(2, b'`')) {
            result = result.cat(lit("\\[ga]"));
            self.consume(b'`');
            result = result.cat(self.parse_preformatted_literal(b"`"))
        }
        if !self.at_end() {
            self.consume(b'`');
            if !self.at_end() && self.current_char != 4 {
                self.consume(b'`');
                if !self.at_end() && self.current_char != 4 {
                    self.consume(b'`');
                }
            }
        }
        Tree::CodeBlock(Box::new(block_type), Box::new(result))
    }

    fn parse_code(&mut self) -> Tree {
        self.consume(b'`'); // opening quote
                            // here, we need to peek 1 and 2 characters ahead to see if
                            // they are also back ticks, and if so parse a code block
                            // instead of an inline code snippet.
        if self.peek(0, b'`') && self.peek(1, b'`') {
            return self.parse_code_block();
        }

        // this is an ugly groff necessity: if our code snippet
        // begins with a dot, we need to escape it
        let exp = if self.current_char == b'.' {
            self.consume(b'.');
            escape_lit(".")
        } else {
            Tree::Empty()
        };
        let code_exp = self.parse_literal(b"`");
        let exp = match exp {
            Tree::Empty() => code_exp,
            _ => exp.cat(code_exp),
        };
        self.consume(b'`'); // closing quote
        Tree::InlineCode(Box::new(exp))
    }

    fn parse_image_size(&mut self) -> Tree {
        let x = self.parse_complete_until(b"x");
        self.consume(b'x');
        let y = self.parse_complete_until(b"]");
        image_size(x, y)
    }

    fn parse_image(&mut self) -> Tree {
        if self.peek(1, b'[') {
            self.consume(b'!');
            self.consume(b'[');
            let caption = self.parse_complete_until(b"|]");
            let mut size_spec = image_size(lit("100"), lit("100"));
            if self.current_char == b'|' {
                self.consume(b'|');
                size_spec = self.parse_image_size();
            }
            self.consume(b']');
            self.consume(b'(');
            let path = self.parse_literal(b")");
            self.consume(b')');
            image(caption, path, size_spec)
        } else {
            lit("!")
        }
    }

    /// Parses only formatting subset of markup as opposed to global_parse_until
    fn parse_format_until(&mut self, break_chars: &[u8]) -> Tree {
        let mut expression = Tree::Empty(); // we start with
                                            // "nothing", as rust has
                                            // no null values
        while !self.at_end() && !break_chars.contains(&self.current_char) {
            let expr = match self.current_char {
                b'*' => self.try_bold_or_lit_until(b"]"),
                b'_' => Tree::Italic(Box::new(self.parse_symmetric_quoted())),
                b'{' => Tree::SmallCaps(Box::new(self.parse_quoted(b'}'))),
                b'`' => self.parse_code(),
                b'"' => Tree::Quote(Box::new(self.parse_symmetric_quoted())),
                b'^' => self.parse_footnote(),
                b'%' => self.parse_drop_cap(),
                b'&' => {
                    self.consume(self.current_char);
                    escape_lit("&")
                }
                b'.' => {
                    self.consume(self.current_char);
                    escape_lit(".")
                }
                b'/' => self.parse_pass_through(),
                b'\\' => self.parse_color_spec(),
                _ => self.parse_literal(
                    format!("_*#\"^`&[{{{}>\n", str::from_utf8(break_chars).unwrap()).as_bytes(),
                ),
            };
            expression = match expression {
                Tree::Empty() => expr,
                _ => expression.cat(expr),
            };
        }
        expression
    }

    /// parses complete mark-up set, as opposed to only formatting, like above fmt_parse_until.
    /// stops parsing when one of the characters in `break_chars` is encountered.
    fn parse_complete_until(&mut self, break_chars: &[u8]) -> Tree {
        // we start with "nothing", as rust has no null values
        let mut expression = Tree::Empty();
        // main parsing loop. note that this function might be called recursivly.
        while !self.at_end() && !break_chars.contains(&self.current_char) {
            let expr = match self.current_char {
                b'-' => self.parse_mdb_or_list_or_lit(),
                b'#' => self.parse_heading(),
                b'*' => self.parse_list_or_bold_or_lit(0, b'*'),
                b'_' => Tree::Italic(Box::new(self.parse_symmetric_quoted())),
                b'{' => Tree::SmallCaps(Box::new(self.parse_quoted(b'}'))),
                b'`' => self.parse_code(),
                b'"' => Tree::Quote(Box::new(self.parse_symmetric_quoted())),
                b'^' => self.parse_footnote(),
                b'%' => self.parse_drop_cap(),
                b'&' => {
                    self.consume(self.current_char);
                    escape_lit("&")
                }
                b'.' => {
                    self.consume(self.current_char);
                    escape_lit(".")
                }
                b'/' => self.parse_pass_through(),
                b'\\' => self.parse_color_spec(),
                b'[' => self.parse_link(),
                b'\n' => {
                    // if the blank line is followed by a heading do not insert a paragraph
                    if self.peek(1, b'\n') {
                        if self.peek(2, b'#') {
                            // a heading follows
                            self.consume(b'\n');
                            self.consume(b'\n');
                            Tree::LineBreak()
                        } else {
                            // no heading follows
                            self.consume(b'\n');
                            self.consume(b'\n');
                            Tree::LineBreak().cat(Tree::Paragraph())
                        }
                    } else {
                        self.consume(b'\n');
                        Tree::LineBreak()
                    }
                }
                b'>' => self.parse_right_sidenote(),
                b'!' => self.parse_image(),
                _ => self.parse_literal(
                    format!("_*#\"^`&[{{{}>\n", str::from_utf8(break_chars).unwrap()).as_bytes(),
                ),
            };
            expression = match expression {
                Tree::Empty() => expr,
                _ => expression.cat(expr),
            };
        }
        expression
    }
}
#[cfg(test)]
mod tests {
    use super::Parser;
    fn parse_to_ast(s: &str) -> String {
        let p = Parser::parse(s);
        format!("{:?}", p)
    }
    #[test]
    fn construction() {
        let parser = Parser::new("\"quoted\"");
        assert_eq!(format!("{:?}", parser), "Parser { input: [34, 113, 117, 111, 116, 101, 100, 34], input_len: 8, current_position: 0, current_line: 1, current_char: 34, doc_type: \"\" }");
    }
    #[test]
    fn expression() {
        assert_eq!(
            parse_to_ast("\"quoted\""),
            "Document(DEFAULT, Quote(Literal(\"quoted\")))"
        );
    }
    #[test]
    fn ampersand() {
        assert_eq!(parse_to_ast("&"), "Document(DEFAULT, EscapeLit(\"&\"))");
    }
    #[test]
    fn dot() {
        assert_eq!(parse_to_ast("."), "Document(DEFAULT, EscapeLit(\".\"))");
    }
    #[test]
    fn wrong_nesting() {
        assert_eq!(
            parse_to_ast("*this is _wrong* nesting_"),
            "Document(DEFAULT)"
        )
    }
}
