use std::env;

use std::fs;

use std::time::Instant;

use mato::config::Config;
use mato::die;
use mato::establish_log_level;
use mato::mato_dbg;
use mato::mato_inf;
use mato::mato_trc;
use mato::opt_flag;
use mato::opts;
use mato::process::canonicalize;
use mato::process::chain;
use mato::process::chain::Chain;
use mato::process::code_block;
use mato::process::image_converter;
use mato::render::groff;
use mato::watch;

const TARGET_FILE_EXTENSION_PDF: &str = "pdf";
const TARGET_FILE_EXTENSION_GRO: &str = "groff";

fn main() -> std::io::Result<()> {
    let mut config = Config::default();
    let mut p = opts::Parser::new();
    p.add_opt(opt_flag!(
        "w",
        "watch",
        "watch file for changes and retransform"
    ));
    p.add_opt(opt_flag!(
        "g",
        "dump-groff",
        "Dump generated groff to standard out."
    ));
    p.add_opt(opt_flag!(
        "G",
        "dump-groff-file",
        "Dump generated groff to file <input>.groff."
    ));
    p.add_opt(opt_flag!(
        "Z",
        "skip-render-and-dump",
        "Skip rendering and dumps groff output."
    ));

    let parsed_opts = p.parse(env::args().collect());
    parsed_opts.handle_standard_flags("matopdf", "0.1.1");
    mato::log::set_log_level(establish_log_level(&parsed_opts));

    // TODO support multiple markdown input files
    if parsed_opts.params.len() < 1 {
        die!("no markdown input file provided! please provide one.");
    }
    config.source_file = parsed_opts.params.first().unwrap().clone();

    mato_dbg!("source file:\t\t{}", &config.source_file);

    config.lang = parsed_opts.get_opt("lang");

    config.watch = parsed_opts.get_flag("watch");

    config.dump_groff = parsed_opts.get_flag("dump-groff");
    config.dump_groff_file = parsed_opts.get_flag("dump-groff-file");
    if parsed_opts.get_flag("skip-render-and-dump") {
        config.skip_rendering = true;
        config.dump_groff = true;
    }
    mato_dbg!("config: {:#?}", config);

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

fn create_chain() -> Chain {
    mato_trc!("constructing chain...");
    let chain = chain::new(canonicalize::new(), image_converter::new()).append(code_block::new());
    mato_trc!("done");
    mato_dbg!("chain: {:?}", chain);
    chain
}

fn matopdf(config: &Config) {
    let input = mato::read_input(&config.source_file);

    let mut chain = create_chain();

    // MD -> GROFF
    let start = Instant::now();
    let groff_output = mato::transform(&mut groff::mom::new(config), &mut chain, config, &input);
    mato_inf!("transformed in:\t\t{:?}", start.elapsed());

    if config.dump_groff {
        println!("{groff_output}");
    }
    if config.dump_groff_file {
        let path_target_file =
            mato::replace_file_extension(&config.source_file, TARGET_FILE_EXTENSION_GRO);
        fs::write(path_target_file, groff_output.clone()).expect("Unable to write groff file");
    }

    let pdf_target_file =
        mato::replace_file_extension(&config.source_file, TARGET_FILE_EXTENSION_PDF);
    // GROFF -> PDF
    if !config.skip_rendering {
        let start = Instant::now();
        let pdf_output = mato::grotopdf(config, &groff_output);
        mato_inf!("groff rendering:\t{:?} ", start.elapsed());

        let start = Instant::now();
        fs::write(&pdf_target_file, pdf_output).expect("Unable to write output pdf");
        mato_inf!("written in:\t\t{:?} ", start.elapsed());
    }
}

#[cfg(test)]
mod tests {
    use mato::config::Config;

    fn matogro(input: &str) -> String {
        let mut config = Config::default();
        config.skip_preamble = true;
        let mut chain = super::create_chain();
        mato::transform(
            &mut super::groff::mom::new(&config),
            &mut chain,
            &config,
            input,
        )
    }

    #[test]
    fn literal() {
        assert_eq!(matogro("hallo"), ".DOCTYPE DEFAULT\n.START\nhallo");
    }
    #[test]
    fn italic() {
        assert_eq!(
            matogro("_hallo_"),
            ".DOCTYPE DEFAULT\n.START\n\\*[IT]hallo\\*[ROM]"
        );
    }
    #[test]
    fn bold() {
        assert_eq!(
            matogro("*hallo*"),
            ".DOCTYPE DEFAULT\n.START\n\\*[BD]hallo\\*[ROM]"
        );
    }

    #[test]
    fn complex_code() {
        assert_eq!(
            matogro("`    -P /opt/homebrew/Cellar/groff/1.22.4_1/share/groff/`"),
            ".DOCTYPE DEFAULT\n.START\n\\*[CODE]    -P /opt/homebrew/Cellar/groff/1.22.4_1/share/groff/\\*[CODE OFF]"
        );
    }

    #[test]
    fn link() {
        assert_eq!(
            matogro("some text [link text](http://example.com)"),
            ".DOCTYPE DEFAULT\n.START\nsome text \\c\n.PDF_WWW_LINK http://example.com \"link text\"\\c\n"
        );
    }
    #[test]
    fn not_link() {
        assert_eq!(
            matogro("some text [link text]"),
            ".DOCTYPE DEFAULT\n.START\nsome text [link text]"
        );
    }

    #[test]
    fn heading_and_subheading() {
        assert_eq!(
            matogro(
                "# heading\n\n## subheading"
            ),
            ".DOCTYPE DEFAULT\n.START\n.FT B\n.EW 2\n.HEADING 1 \"heading\"\n.EW 0\n.FT R\n.DRH\n.FT B\n.EW 2\n.HEADING 2 \"subheading\"\n.EW 0\n.FT R"
        );
    }

    #[test]
    fn heading_and_paragraph() {
        assert_eq!(
            matogro("# heading\n\nA new paragraph"),
            ".DOCTYPE DEFAULT\n.START\n.FT B\n.EW 2\n.HEADING 1 \"heading\"\n.EW 0\n.FT R\n.DRH\n.SP 1v\n.PP\nA new paragraph"
        );
    }
    #[test]
    fn paragraph_and_heading() {
        assert_eq!(
            matogro("A new paragraph\n\n# heading"),
            ".DOCTYPE DEFAULT\n.START\nA new paragraph\n.FT B\n.EW 2\n.HEADING 1 \"heading\"\n.EW 0\n.FT R\n.DRH"
        );
    }

    #[test]
    fn code_block() {
        assert_eq!(
            matogro("```\nPP\n```\n"),
            //            ".QUOTE_STYLE INDENT 1\n.QUOTE\n.CODE\n.BOX OUTLINED black INSET 18p\nPP\n.BOX OFF\n.QUOTE OFF"
            ".DOCTYPE DEFAULT\n.START\n.QUOTE_STYLE INDENT 1\n.QUOTE\n.CODE\nPP\n.QUOTE OFF\n"
        );
    }
    #[test]
    fn code_escape_literal() {
        assert_eq!(
            matogro("`.PP`"),
            ".DOCTYPE DEFAULT\n.START\n\\*[CODE]\\&.PP\\*[CODE OFF]"
        );
    }
    #[test]
    fn chapter_mark() {
        assert_eq!(
            matogro(">>(c)\n"),
            ".DOCTYPE DEFAULT\n.START\n.MN RIGHT\n.PT_SIZE +48\nc\n.MN OFF"
        );
    }
    #[test]
    fn not_chapter_mark() {
        assert_eq!(matogro(">>c"), ".DOCTYPE DEFAULT\n.START\n>>c");
    }
    #[test]
    fn right_side_note() {
        assert_eq!(
            matogro(">(side)\n"),
            ".DOCTYPE DEFAULT\n.START\n\n.MN RIGHT\n.PT_SIZE -2\nside\n.MN OFF\n\n"
        );
    }
    #[test]
    fn not_right_side_note() {
        assert_eq!(matogro(">side"), ".DOCTYPE DEFAULT\n.START\n>side");
    }
    #[test]
    fn foot_note() {
        assert_eq!(
            matogro("^(side)\n"),
            ".DOCTYPE DEFAULT\n.START\n\\c\n.FOOTNOTE\nside\n.FOOTNOTE END\n\n"
        );
    }
    #[test]
    fn list_1() {
        assert_eq!(
            matogro("* list item\n"),
            ".DOCTYPE DEFAULT\n.START\n.LIST\n.SHIFT_LIST 18p\n.ITEM\nlist item\n.LIST OFF\n"
        );
    }
    #[test]
    fn list_2() {
        assert_eq!(
            matogro("* list item 1\n* list item 2\n"),
            ".DOCTYPE DEFAULT\n.START\n.LIST\n.SHIFT_LIST 18p\n.ITEM\nlist item 1\n.ITEM\nlist item 2\n.LIST OFF\n"
        );
    }
    #[test]
    fn nested_list() {
        assert_eq!(matogro("* list item 1\n  * list item 2\n"), ".DOCTYPE DEFAULT\n.START\n.LIST\n.SHIFT_LIST 18p\n.ITEM\nlist item 1\n.LIST\n.SHIFT_LIST 18p\n.ITEM\nlist item 2\n.LIST OFF\n.LIST OFF\n");
    }
    #[test]
    fn list_1_multiline_item() {
        assert_eq!(
            matogro("* list item\n  which continues on next line\n"),
            ".DOCTYPE DEFAULT\n.START\n.LIST\n.SHIFT_LIST 18p\n.ITEM\nlist item\nwhich continues on next line\n.LIST OFF\n"
        );
    }
}
