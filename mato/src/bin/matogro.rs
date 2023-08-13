use std::env;

use std::fs;
use std::io::Write;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

use mato::config::Config;
use mato::render::groff;
use mato::watch;

fn main() -> std::io::Result<()> {
    let config = Config::from(env::args().collect());

    let mut mom_preamble = include_str!("default-preamble.mom").to_string();

    // try to find preamble.mom located next to source file
    let path_source_file = Path::new(&config.source_file);
    let parent_dir = path_source_file
        .parent()
        .expect("could not establish parent path of file");
    let sibbling_preamble = parent_dir.join("preamble.mom");
    if sibbling_preamble.as_path().is_file() {
        println!("found sibbling preamble: {}", sibbling_preamble.display());
        mom_preamble = fs::read_to_string(sibbling_preamble)?;
    } else {
        println!("preamble:\t\tbuilt-in");
    }
    if config.dump {
        println!("{mom_preamble}");
    }

    // open source file to be able watch it (we need a file descriptor)
    println!("source file:\t\t{}", &config.source_file);

    let mut path_target_file = path_source_file.to_path_buf();
    path_target_file.set_extension("pdf");
    println!("target file name:\t{}", path_target_file.display());

    if config.watch {
        let kqueue = watch::Kqueue::create();
        loop {
            transform_and_render(
                &config,
                &config.source_file,
                path_target_file.to_str().unwrap(),
                &mom_preamble,
            );
            kqueue.wait_for_write_on_file_name(&config.source_file)?;
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
    mato::transform(&groff::Renderer {}, input)
}

fn grotopdf(input: &str, mom_preamble: &str) -> Vec<u8> {
    let mut child = Command::new("/usr/bin/env")
        .arg("pdfmom")
        .arg("-mden")
        .args(["-K", "UTF-8"]) // process with preconv to support utf-8
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
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
    if !output.stderr.is_empty() {
        let _ = io::stderr().write(&output.stderr);
    }
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
        println!("{groff_output}");
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
    use super::matogro;

    #[test]
    fn literal() {
        assert_eq!(matogro("hallo"), "hallo");
    }
    #[test]
    fn italic() {
        assert_eq!(matogro("_hallo_"), "\\*[IT]hallo\\*[ROM]");
    }
    #[test]
    fn bold() {
        assert_eq!(matogro("*hallo*"), "\\*[BD]hallo\\*[ROM]");
    }

    #[test]
    fn complex_code() {
        assert_eq!(
            matogro("`    -P /opt/homebrew/Cellar/groff/1.22.4_1/share/groff/`"),
            "\\*[CODE]    -P /opt/homebrew/Cellar/groff/1.22.4_1/share/groff/\\*[CODE OFF]"
        );
    }

    #[test]
    fn link() {
        assert_eq!(
            matogro("some text [link text](http://example.com)"),
            "some text .PDF_WWW_LINK http://example.com \"link text\""
        );
    }
    #[test]
    fn not_link() {
        assert_eq!(matogro("some text [link text]"), "some text [link text]");
    }

    #[test]
    fn heading_and_subheading() {
        assert_eq!(
            matogro(
                "# heading\n\n## subheading"
            ),
            ".SPACE -.7v\n.EW 2\n.HEADING 1 \"heading\"\n.EW 0\n\n.SPACE -.7v\n.EW 2\n.HEADING 2 \"subheading\"\n.EW 0\n"
        );
    }

    #[test]
    fn heading_and_paragraph() {
        assert_eq!(
            matogro("# heading\n\nA new paragraph"),
            ".SPACE -.7v\n.EW 2\n.HEADING 1 \"heading\"\n.EW 0\n\n.PP\nA new paragraph"
        );
    }
    #[test]
    fn paragraph_and_heading() {
        assert_eq!(
            matogro("A new paragraph\n\n# heading"),
            "A new paragraph\n\n.SPACE -.7v\n.EW 2\n.HEADING 1 \"heading\"\n.EW 0\n"
        );
    }

    #[test]
    fn code_block() {
        assert_eq!(
            matogro("```\nPP\n```\n"),
            ".QUOTE_STYLE INDENT 1\n.QUOTE\n.CODE\n.BOX OUTLINED black INSET 18p\nPP\n.BOX OFF\n.QUOTE OFF"
        );
    }
    #[test]
    fn code_escape_literal() {
        assert_eq!(matogro("`.PP`"), "\\*[CODE]\\&.PP\\*[CODE OFF]");
    }
    #[test]
    fn chapter_mark() {
        assert_eq!(
            matogro(">>(c)\n"),
            ".MN RIGHT\n.PT_SIZE +48\n.COLOR grey\nc\n.MN OFF\n"
        );
    }
    #[test]
    fn not_chapter_mark() {
        assert_eq!(matogro(">>c"), ">>c");
    }
    #[test]
    fn right_side_note() {
        assert_eq!(
            matogro(">(side)\n"),
            "\n.MN RIGHT\n.PT_SIZE -2\nside\n.MN OFF\n\n"
        );
    }
    #[test]
    fn not_right_side_note() {
        assert_eq!(matogro(">side"), ">side");
    }
    #[test]
    fn foot_note() {
        assert_eq!(matogro("^(side)\n"), "\n.FOOTNOTE\nside\n.FOOTNOTE END\n\n");
    }
}
