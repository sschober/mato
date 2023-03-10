use std::env;

use mato::renderer::tex::TexRenderer;


fn main() {
    for file in env::args().skip(1) {
        let input = std::fs::read_to_string(file).unwrap();
        println!("{}", mato::transform(TexRenderer{}, input.as_str()));
    }
}

#[cfg(test)]
mod tests {
    use mato::renderer::tex::TexRenderer;


    #[test]
    fn literal() {
        assert_eq!(mato::transform(TexRenderer{}, "hallo"), "hallo");
    }
    #[test]
    fn italic() {
        assert_eq!(mato::transform(TexRenderer{}, "_hallo_"), "\\textit{hallo}");
    }
    #[test]
    fn bold() {
        assert_eq!(mato::transform(TexRenderer{}, "*hallo*"), "\\textbf{hallo}");
    }
    /*
    #[test]
    fn heading() {
        assert_eq!(
            super::Parser::transform("# heading\n"),
            "\\section{heading}\n"
        );
    }
    #[test]
    fn heading_without_newline() {
        assert_eq!(super::Parser::transform("# 1"), "\\section{1}");
    }
    #[test]
    fn quote() {
        assert_eq!(super::Parser::transform("\"input\""), "\"`input\"'");
    }
    #[test]
    fn bold_and_italic() {
        assert_eq!(
            super::Parser::transform("*_text_*"),
            "\\textbf{\\textit{text}}"
        );
    }
    #[test]
    fn bold_and_italic_but_with_outer_chars() {
        assert_eq!(
            super::Parser::transform("*fett _kursiv_ wieder fett*"),
            "\\textbf{fett \\textit{kursiv} wieder fett}"
        );
    }
    
    #[test]
    fn footnote() {
        assert_eq!(
            super::Parser::transform("input^(footnote)"),
            "input~\\footnote{footnote}"
        );
    }
    
    #[test]
    fn teletype(){
        assert_eq!(super::Parser::transform("`input`"), "\\texttt{input}");
    }
    
    #[test]
    fn ampersand_is_escaped(){
        assert_eq!(super::Parser::transform("&"), "\\&");
    }
    
    #[test]
    fn link(){
        assert_eq!(super::Parser::transform("[link text](http://example.com)"), "\\href{http://example.com}{link text}");
    }
    
    #[test]
    fn brackets_are_kept(){
        assert_eq!(super::Parser::transform("[link text]"), "[link text]");
    }
    */
}
