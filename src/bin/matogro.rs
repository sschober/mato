use std::env;

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

use mato::renderer::groff::GroffRenderer;
use mato::watch;
use mato::config::Config;

fn main() -> std::io::Result<()> {
    let config = Config::from(env::args());

    let mut mom_preamble = include_str!("default-preamble.mom").to_string();

    // try to find preamble.mom located next to source file
    let path_source_file = Path::new(&config.source_file);
    let parent_dir = path_source_file
        .parent()
        .expect("could not establish parent path of file");
    let sibbling_preamble = parent_dir.join("preamble.mom");
    if sibbling_preamble.as_path().is_file() {
        println!("found sibbling preamble: {}", sibbling_preamble.display());
        mom_preamble = fs::read_to_string(sibbling_preamble)?
    }
    println!("using preamble:\n{}", mom_preamble);

    // open source file to be able watch it (we need a file descriptor)
    println!("source file:\t\t{}", &config.source_file);

    let mut path_target_file = path_source_file.to_path_buf();
    path_target_file.set_extension("pdf");
    println!("target file name:\t{}", path_target_file.display());

    if config.watch {
        let kqueue = watch::Kqueue::create();
        loop {
            kqueue.wait_for_write_on_file_name(&config.source_file)?;
            transform_and_render(
                &config,
                &config.source_file,
                path_target_file.to_str().unwrap(),
                &mom_preamble,
            );
        }
    } else {
        transform_and_render(
            &config,
            &config.source_file,
            path_target_file.to_str().unwrap(),
            &mom_preamble,
        );
    };
    Ok(())
}

fn matogro(input: &str) -> String {
    mato::transform(GroffRenderer {}, input)
}

fn grotopdf(input: &str, mom_preamble: &str) -> Vec<u8> {
    let mut child = Command::new("/opt/homebrew/bin/pdfmom")
        .arg("-mden")
        .arg("-K UTF-8") // process with preconv to support utf-8
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn pdfmom");

    {
        // this lexical block is only here to let stdin run out of scope to be closed...
        let mut stdin = child.stdin.take().expect("Failed to open stdin for pdfmom");
        stdin
            .write_all(mom_preamble.as_bytes())
            .expect("Failed to write preamble to stdin");
        stdin
            .write_all(input.as_bytes())
            .expect("Failed to write to stdin of pdfmom");
    }
    // ... otherwise this call would not terminate
    let output = child.wait_with_output().expect("Failed to read stdout");
    output.stdout
}

    fn transform_and_render(config: &Config, source_file: &str, target_file: &str, mom_preamble: &str) {
        let start = Instant::now();
        let input = std::fs::read_to_string(source_file).unwrap();
        println!("read in:\t\t{:?}", start.elapsed());

        let start = Instant::now();
        let groff_output = matogro(&input);
        println!("transformed in:\t\t{:?}", start.elapsed());
        if config.dump {
            println!("{}", groff_output);
        }

        let start = Instant::now();
        let pdf_output = grotopdf(&groff_output, mom_preamble);
        println!("groff rendering:\t{:?} ", start.elapsed());

        let start = Instant::now();
        fs::write(target_file, pdf_output).expect("Unable to write out.pdf");
        println!("written in:\t\t{:?} ", start.elapsed());
    }

#[cfg(test)]
mod tests {
    use mato::renderer::groff::GroffRenderer;

    #[test]
    fn literal() {
        assert_eq!(mato::transform(GroffRenderer {}, "hallo"), "hallo");
    }
    #[test]
    fn italic() {
        assert_eq!(
            mato::transform(GroffRenderer {}, "_hallo_"),
            "\\*[IT]hallo\\*[ROM]"
        );
    }
    #[test]
    fn bold() {
        assert_eq!(
            mato::transform(GroffRenderer {}, "*hallo*"),
            "\\*[BD]hallo\\*[ROM]"
        );
    }

    #[test]
    fn complex_code() {
        assert_eq!(
            mato::transform(
                GroffRenderer {},
                "`    -P /opt/homebrew/Cellar/groff/1.22.4_1/share/groff/`"
            ),
            "\\*[CODE]    -P /opt/homebrew/Cellar/groff/1.22.4_1/share/groff/\\*[CODE OFF]"
        );
    }

    #[test]
    fn link(){
        assert_eq!(
            mato::transform(
                GroffRenderer {},
                "some text [link text](http://example.com)"
            ), 
            "some text .PDF_WWW_LINK http://example.com \"link text\""
        );
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
    fn brackets_are_kept(){
        assert_eq!(super::Parser::parse("[link text]"), "[link text]");
    }

    */
}
