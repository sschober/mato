use std::env;

use mato::renderer::groff::GroffRenderer;

fn main() {
    let mom_preamble = include_str!("default-preamble.mom");
    println!("{}", mom_preamble);

    for file in env::args().skip(1) {
        let input = std::fs::read_to_string(file).unwrap();
        println!("{}", mato::transform(GroffRenderer{}, input.as_str()));
    }
}

#[cfg(test)]
mod tests {
    use mato::renderer::groff::GroffRenderer;


    #[test]
    fn literal() {
        assert_eq!(mato::transform(GroffRenderer{}, "hallo"), "hallo");
    }
    #[test]
    fn italic() {
        assert_eq!(mato::transform(GroffRenderer{}, "_hallo_"), "\\*[SLANT]hallo\\*[SLANTX]");
    }
    #[test]
    fn bold() {
        assert_eq!(mato::transform(GroffRenderer{}, "*hallo*"), "\\*[BOLDER]hallo\\*[BOLDERX]");
    }
    /*
    #[test]
    fn heading() {
        assert_eq!(
            super::Parser::parse("# heading\n"),
            "\\section{heading}\n"
        );
    }
    #[test]
    fn heading_without_newline() {
        assert_eq!(super::Parser::parse("# 1"), "\\section{1}");
    }
    #[test]
    fn quote() {
        assert_eq!(super::Parser::parse("\"input\""), "\"`input\"'");
    }
    #[test]
    fn bold_and_italic() {
        assert_eq!(
            super::Parser::parse("*_text_*"),
            "\\textbf{\\textit{text}}"
        );
    }
    #[test]
    fn bold_and_italic_but_with_outer_chars() {
        assert_eq!(
            super::Parser::parse("*fett _kursiv_ wieder fett*"),
            "\\textbf{fett \\textit{kursiv} wieder fett}"
        );
    }
    
    #[test]
    fn footnote() {
        assert_eq!(
            super::Parser::parse("input^(footnote)"),
            "input~\\footnote{footnote}"
        );
    }
    
    #[test]
    fn teletype(){
        assert_eq!(super::Parser::parse("`input`"), "\\texttt{input}");
    }
    
    #[test]
    fn ampersand_is_escaped(){
        assert_eq!(super::Parser::parse("&"), "\\&");
    }
    
    #[test]
    fn link(){
        assert_eq!(super::Parser::parse("[link text](http://example.com)"), "\\href{http://example.com}{link text}");
    }
    
    #[test]
    fn brackets_are_kept(){
        assert_eq!(super::Parser::parse("[link text]"), "[link text]");
    }
    */
}
