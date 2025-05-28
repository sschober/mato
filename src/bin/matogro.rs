use std::env;

use mato::{
    config::Config, create_default_chain, die, establish_log_level, mato_dbg, mato_trc, opt_flag,
    opt_val, opts, process::chain::Chain, render::groff, Render,
};

/// command close to groffs idea, which renders output to
/// standard out and is configured via `-T<device>`, with
/// a default of `mom`.
fn main() -> std::io::Result<()> {
    let mut config = Config::default();
    let mut p = opts::Parser::new();
    let opt_dump_dot_file = p.add_opt(opt_flag!(
        "X",
        "dump-dot-file",
        "Write graphviz dot representation of parse tree to file."
    ));
    let opt_lang = p.add_opt(opt_val!("l", "lang", "Set document language.", "den"));
    let opt_device = p.add_opt(opt_val!(
        "T",
        "device",
        "Backend device to be used for rendering.",
        "mom"
    ));

    let parsed_opts = p.parse(env::args().collect());
    parsed_opts.handle_standard_flags("mato", "0.1.1");
    mato::log::set_log_level(establish_log_level(&parsed_opts));
    if parsed_opts.params.is_empty() {
        die!("no markdown input file provided! please provide one.");
    }

    config.lang = opt_lang.val(&parsed_opts);

    config.dump_dot_file = opt_dump_dot_file.is_set(&parsed_opts);

    config.source_file = parsed_opts.params.first().unwrap().clone();
    mato_dbg!("source file:\t\t{}", &config.source_file);

    mato_trc!("{:?}", config);

    let mut render: Box<dyn Render + '_>;
    let mut chain: Chain;
    let device = opt_device.val(&parsed_opts);
    match device.as_str() {
        "mom" => {
            chain = create_default_chain(&config, true);
            render = Box::new(groff::mom::new(&config));
        }
        "man" => {
            chain = create_default_chain(&config, true);
            render = Box::new(groff::man::new());
        }
        "mdoc" => {
            chain = create_default_chain(&config, false);
            render = Box::new(groff::mandoc::new());
        }
        _ => {
            die!("Unknown device: {}", device);
        }
    };
    let input = mato::read_input(&config.source_file);
    println!(
        "{}",
        mato::transform(&mut render, &mut chain, &config, &input)
    );
    Ok(())
}
