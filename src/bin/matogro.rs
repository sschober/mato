use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::process::{Command, Stdio};
use std::ptr;
use std::time::Instant;

use mato::renderer::groff::GroffRenderer;

mod ffi {

    pub const EVFILT_VNODE: i16 = -4;
    pub const EV_ADD: u16 = 0x1;
    pub const EV_ENABLE: u16 = 0x4;
    pub const EV_CLEAR: u16 = 0x20;

    pub const NOTE_WRITE: u32 = 0x00000002;

    #[derive(Debug)]
    #[repr(C)]
    pub(super) struct Timespec {
        /// Seconds
        tv_sec: isize,
        /// Nanoseconds
        v_nsec: usize,
    }

    #[derive(Debug, Clone, Default)]
    #[repr(C)]
    pub struct Kevent {
        pub ident: u64,
        pub filter: i16,
        pub flags: u16,
        pub fflags: u32,
        pub data: i64,
        pub udata: u64,
    }

    #[link(name = "c")]
    extern "C" {
        pub(super) fn kqueue() -> i32;
        pub(super) fn kevent(
            kq: i32,
            changelist: *const Kevent,
            nchanges: i32,
            eventlist: *mut Kevent,
            nevents: i32,
            timeout: *const Timespec,
        ) -> i32;
    }
}

fn main() -> std::io::Result<()> {

    let mom_preamble = include_str!("default-preamble.mom");
    // TODO implement sane preamble logic
    // if exists a .preamble.mom in current dir => use that
    // if exists a ~/.mato/preamble.mom => use that
    // => use default
    println!("using preamble:\n{}", mom_preamble);
    let file = env::args().skip(1).nth(0).expect("need a file as argument");

    println!("opening file {}", &file);
    let f = File::open(&file)?;
    let fd = f.as_raw_fd();
    println!("got fd: {}", fd);

    println!("creating kqueue...");
    let queue = unsafe { ffi::kqueue() };
    if queue < 0 {
        panic!("{}", std::io::Error::last_os_error());
    }
    println!("kqueue: {} ... looping", queue);

    loop {
        let event = ffi::Kevent {
            ident: fd as u64,
            filter: ffi::EVFILT_VNODE,
            flags: ffi::EV_ADD | ffi::EV_ENABLE | ffi::EV_CLEAR,
            fflags: ffi::NOTE_WRITE,
            data: 0,
            udata: 0,
        };
        let mut changelist = [event];
        println!("constructed changelist... calling kevent...");
        let res = unsafe {
            ffi::kevent(
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
        transform_and_render(file.clone(), mom_preamble);
    }
}

fn transform_and_render(file: String, mom_preamble: &str) {
    let start = Instant::now();

    let input = std::fs::read_to_string(file).unwrap();
    let groff_output = mato::transform(GroffRenderer {}, input.as_str());
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
    fs::write("out.pdf", output.stdout).expect("Unable to write out.pdf");
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