use std::env;

use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

use mato::config::Config;
use mato::process::canonicalize;
use mato::process::chain;
use mato::process::code_block;
use mato::process::image_converter;
use mato::process::meta_data_extractor;
use mato::render::groff;
use mato::watch;

fn main() -> std::io::Result<()> {
    let config = Config::from(env::args().collect());
    eprintln!("config: {:#?}", config);
    let mut mom_preamble = include_str!("default-preamble.mom").to_string();

    // try to find preamble.mom located next to source file
    let sibbling_preamble = Path::new(&config.parent_dir).join("preamble.mom");
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
    if !Path::new(&config.source_file).exists() {
        eprintln!("Could not open source file: {}", config.source_file);
        std::process::exit(1);
    }
    let path_source_file = Path::new(&config.source_file);
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

fn matogro_with_config(input: &str, config: &Config, mom_preamble: &str) -> String {
    let mde = meta_data_extractor::MetaDataExtractor::from(mom_preamble);
    let canon = canonicalize::Canonicalizer {};
    let code_block_proc = code_block::CodeBlockProcessor {};
    let chain = chain::Chain {
        a: Box::new(canon),
        b: Box::new(mde),
    };
    let chain_outer = chain::Chain {
        a: Box::new(chain),
        b: Box::new(image_converter::ImageConverter {}),
    };
    let mut chain_outer2 = chain::Chain {
        a: Box::new(chain_outer),
        b: Box::new(code_block_proc),
    };
    mato::transform(
        &mut groff::Renderer::new(),
        &mut chain_outer2,
        config,
        input,
    )
}

fn grotopdf(config: &Config, input: &str) -> Vec<u8> {
    let mut child = Command::new("/usr/bin/env")
        .arg("pdfmom")
        .arg(format!("-m{}", config.lang))
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
    let groff_output = matogro_with_config(&input, config, mom_preamble);
    println!("transformed in:\t\t{:?}", start.elapsed());
    if config.dump {
        //println!("{groff_output}");
        let path_source_file = Path::new(&config.source_file);
        let mut path_target_file = path_source_file.to_path_buf();
        path_target_file.set_extension("gro");
        fs::write(path_target_file, groff_output.clone()).expect("Unable to write gro");
    }

    let start = Instant::now();
    let pdf_output = grotopdf(config, &groff_output);
    println!("groff rendering:\t{:?} ", start.elapsed());

    let start = Instant::now();
    fs::write(target_file, pdf_output).expect("Unable to write out.pdf");
    println!("written in:\t\t{:?} ", start.elapsed());
}

#[cfg(test)]
mod tests {
    use mato::config::Config;

    use super::matogro_with_config;

    fn matogro(input: &str) -> String {
        matogro_with_config(input, &Config::new(), "")
    }

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
            //            ".QUOTE_STYLE INDENT 1\n.QUOTE\n.CODE\n.BOX OUTLINED black INSET 18p\nPP\n.BOX OFF\n.QUOTE OFF"
            ".QUOTE_STYLE INDENT 1\n.QUOTE\n.CODE\nPP\n.QUOTE OFF"
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
    #[test]
    fn list_1() {
        assert_eq!(
            matogro("* list item\n"),
            ".LIST\n.SHIFT_LIST 18p\n.ITEM\nlist item\n.LIST OFF\n"
        );
    }
    #[test]
    fn list_2() {
        assert_eq!(
            matogro("* list item 1\n* list item 2\n"),
            ".LIST\n.SHIFT_LIST 18p\n.ITEM\nlist item 1\n.ITEM\nlist item 2\n.LIST OFF\n"
        );
    }
    #[test]
    fn nested_list() {
        assert_eq!(matogro("* list item 1\n  * list item 2\n"), ".LIST\n.SHIFT_LIST 18p\n.ITEM\nlist item 1\n.LIST\n.SHIFT_LIST 18p\n.ITEM\nlist item 2\n.LIST OFF\n.LIST OFF\n");
    }
    #[test]
    fn list_1_multiline_item() {
        assert_eq!(
            matogro("* list item\n  which continues on next line\n"),
            ".LIST\n.SHIFT_LIST 18p\n.ITEM\nlist item\nwhich continues on next line\n.LIST OFF\n"
        );
    }
}
