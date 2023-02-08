use std::env;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

use mato::renderer::groff::GroffRenderer;

fn main() {
    let mom_preamble = include_str!("default-preamble.mom");
    // TODO implement sane preamble logic
    // if exists a .preamble.mom in current dir => use that
    // if exists a ~/.mato/preamble.mom => use that
    // => use default
    println!("using preamble:\n{}", mom_preamble);    
    for file in env::args().skip(1) {
        let input = std::fs::read_to_string(file).unwrap();
        let groff_output = mato::transform(GroffRenderer{}, input.as_str());
        println!("transformed...");

        let mut child = Command::new("/opt/homebrew/bin/pdfmom")
        .arg("-mden")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn pdfmom");
        println!("spawned pdfmom...");    

        {
            // this lexical block is only here to let stdin run out of scope to be closed...
            let mut stdin = child.stdin.take().expect("Failed to open stdin for pdfmom");
            stdin.write_all(mom_preamble.as_bytes()).expect("Failed to write preamble to stdin");
            stdin.write_all(groff_output.as_bytes()).expect("Failed to write to stdin of pdfmom");
        }
        println!("wrote to stdin...");
        // ... otherwise this call would not terminate
        let output = child.wait_with_output().expect("Failed to read stdout");
        fs::write("out.pdf", output.stdout).expect("Unable to write out.pdf");
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
