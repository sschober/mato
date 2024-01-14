use std::env;

use std::fs;

use std::time::Instant;

use mato::config::Config;
use mato::log_dbg;
use mato::log_inf;
use mato::log_trc;
use mato::process::canonicalize;
use mato::process::chain;
use mato::process::chain::Chain;
use mato::process::code_block;
use mato::process::image_converter;
use mato::process::meta_data_extractor;
use mato::render::groff;
use mato::watch;

const PREAMBLE_FILE_NAME: &str = "preamble.mom";
const TARGET_FILE_EXTENSION_PDF: &str = "pdf";
const TARGET_FILE_EXTENSION_GRO: &str = "gro";

fn main() -> std::io::Result<()> {
    let mut config = Config::from(env::args().collect())?;
    log_dbg!(config, "config: {:#?}", config);
    log_dbg!(config, "source file:\t\t{}", &config.source_file);

    let default_mom_preamble = include_str!("default-preamble.mom").to_string();
    config.locate_and_load_preamble(PREAMBLE_FILE_NAME, &default_mom_preamble);

    config.set_target_file(TARGET_FILE_EXTENSION_PDF);
    log_dbg!(config, "target file name:\t{}", config.target_file);

    if config.watch {
        let kqueue = watch::Kqueue::create();
        loop {
            matopdf(&config);
            kqueue.wait_for_write_on_file_name(&config.source_file)?;
        }
    } else {
        matopdf(&config);
    };
    Ok(())
}

fn create_chain(config: &Config) -> Chain {
    log_trc!(config, "constructing chain...");
    let chain = chain::new(
        canonicalize::new(),
        meta_data_extractor::new(&config.preamble),
    )
    .append(image_converter::new())
    .append(code_block::new());
    log_trc!(config, "done");
    log_dbg!(config, "{:?}", chain);
    chain
}

fn matopdf(config: &Config) {
    let input = mato::read_input(config);

    let mut chain = create_chain(config);

    // MD -> GROFF
    let start = Instant::now();
    let groff_output = mato::transform(&mut groff::new(), &mut chain, config, &input);
    log_inf!(config, "transformed in:\t\t{:?}", start.elapsed());

    if config.dump_groff {
        println!("{groff_output}");
    }
    if config.dump_groff_file {
        let path_target_file = config.target_file(TARGET_FILE_EXTENSION_GRO);
        fs::write(path_target_file, groff_output.clone()).expect("Unable to write gro");
    }

    // GROFF -> PDF
    if !config.skip_rendering {
        let start = Instant::now();
        let pdf_output = mato::grotopdf(config, &groff_output);
        log_inf!(config, "groff rendering:\t{:?} ", start.elapsed());

        let start = Instant::now();
        fs::write(&config.target_file, pdf_output).expect("Unable to write output pdf");
        log_inf!(config, "written in:\t\t{:?} ", start.elapsed());
    }
}

#[cfg(test)]
mod tests {
    use mato::config::Config;

    fn matogro(input: &str) -> String {
        let config = Config::default();
        let mut chain = super::create_chain(&config);
        mato::transform(&mut super::groff::new(), &mut chain, &config, input)
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
            ".SPACE -.7v\n.FT B\n.EW 2\n.HEADING 1 \"heading\"\n.EW 0\n.FT R\n.DRH\n\n.SPACE -.7v\n.FT B\n.EW 2\n.HEADING 2 \"subheading\"\n.EW 0\n.FT R\n"
        );
    }

    #[test]
    fn heading_and_paragraph() {
        assert_eq!(
            matogro("# heading\n\nA new paragraph"),
            ".SPACE -.7v\n.FT B\n.EW 2\n.HEADING 1 \"heading\"\n.EW 0\n.FT R\n.DRH\n\n.PP\nA new paragraph"
        );
    }
    #[test]
    fn paragraph_and_heading() {
        assert_eq!(
            matogro("A new paragraph\n\n# heading"),
            "A new paragraph\n\n.SPACE -.7v\n.FT B\n.EW 2\n.HEADING 1 \"heading\"\n.EW 0\n.FT R\n.DRH\n"
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
