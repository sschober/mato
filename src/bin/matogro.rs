use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::process::{Command, Stdio};
use std::ptr;
use std::time::Instant;

use mato::renderer::groff::GroffRenderer;
use mato::watch;
use mato::watch::Kevent;

fn main() -> std::io::Result<()> {

    let mut mom_preamble = include_str!("default-preamble.mom").to_string();
    // TODO implement sane preamble logic
    // if exists a .preamble.mom in current dir => use that
    // if exists a ~/.mato/preamble.mom => use that
    // => use default
    let file = env::args().skip(1).nth(0).expect("need a file as argument");

    // try to find preamble.mom located next to source file
    let path = Path::new(&file);
    let parent_dir = path.parent().expect("could not establish parent path of file");
    let sibbling_preamble = parent_dir.join("preamble.mom");
    if sibbling_preamble.as_path().is_file() {
        println!("found sibbling preamble: {}", sibbling_preamble.display());
        mom_preamble = fs::read_to_string(sibbling_preamble)?
    }
    println!("using preamble:\n{}", mom_preamble);

    // open source file to be able watch it (we need a file descriptor)
    println!("opening file {}", &file);
    let f = File::open(&file)?;
    let fd = f.as_raw_fd();
    println!("got fd: {}", fd);
    
    let file_stem = path.file_stem().expect("Could not get file stemp").to_str().expect("could not get utf-8 string");
    println!("source file stem: {}", file_stem);
    let target_file_name = format!("{}.pdf", file_stem);
    println!("target file name: {}", target_file_name);

    println!("creating kqueue...");
    let queue = unsafe { watch::kqueue() };
    if queue < 0 {
        panic!("{}", std::io::Error::last_os_error());
    }
    println!("kqueue: {} ... looping", queue);

    loop {
        let event = Kevent::wait_for_write_on(fd);
        let mut changelist = [event];
        println!("constructed changelist... calling kevent...");
        let res = unsafe {
            watch::kevent(
                queue,
                changelist.as_ptr(),
                1,
                changelist.as_mut_ptr(),
                1,
                ptr::null(),
            )
        };
        if res < 0 {
            panic!("{}", std::io::Error::last_os_error());
        }
        println!("...and am back... rending...");
        transform_and_render(file.clone(), target_file_name.clone(), &mom_preamble);
    }
}

fn transform_and_render(source_file: String, target_file: String, mom_preamble: &str) {
    let start = Instant::now();

    let input = std::fs::read_to_string(source_file).unwrap();
    let groff_output = mato::transform(GroffRenderer {}, input.as_str());
    println!("transformed...");

    let mut child = Command::new("/opt/homebrew/bin/pdfmom")
        .arg("-mden")
        .arg("-K UTF-8") // process with preconv to support utf-8
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn pdfmom");
    println!("spawned pdfmom...");

    {
        // this lexical block is only here to let stdin run out of scope to be closed...
        let mut stdin = child.stdin.take().expect("Failed to open stdin for pdfmom");
        stdin
            .write_all(mom_preamble.as_bytes())
            .expect("Failed to write preamble to stdin");
        stdin
            .write_all(groff_output.as_bytes())
            .expect("Failed to write to stdin of pdfmom");
    }
    println!("wrote to stdin...");
    // ... otherwise this call would not terminate
    let output = child.wait_with_output().expect("Failed to read stdout");
    fs::write(target_file, output.stdout).expect("Unable to write out.pdf");
    let duration = start.elapsed();
    println!("total time: {:?} ", duration);
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
            "\\*[SLANT]hallo\\*[SLANTX]"
        );
    }
    #[test]
    fn bold() {
        assert_eq!(
            mato::transform(GroffRenderer {}, "*hallo*"),
            "\\*[BOLDER]hallo\\*[BOLDERX]"
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
    fn link(){
        assert_eq!(super::Parser::parse("[link text](http://example.com)"), "\\href{http://example.com}{link text}");
    }

    #[test]
    fn brackets_are_kept(){
        assert_eq!(super::Parser::parse("[link text]"), "[link text]");
    }

    */
}