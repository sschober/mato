use std::env;

use std::path::Path;
use std::time::Instant;

use mato::config::Config;
use mato::process::identity;
use mato::render::markdown;
use mato::{die, mato_dbg, mato_trc, Render};

fn main() -> std::io::Result<()> {
    let config = Config::from(env::args().collect())?;
    mato_trc!("config: {:#?}", config);

    // open source file to be able watch it (we need a file descriptor)
    mato_dbg!("source file:\t\t{}", &config.source_file);
    if !config.source_file.is_empty() && !Path::new(&config.source_file).exists() {
        die!("Could not open source file: {}", config.source_file);
    }
    let input = mato::read_input(&config.source_file);
    println!("{}", matofmt(&config, &input));
    Ok(())
}

fn matofmt(config: &Config, input: &str) -> String {
    let start = Instant::now();
    let mut processor = identity::Identity {};
    let mut renderer: Box<dyn Render + '_> = Box::new(markdown::Renderer::new());
    let output = mato::transform(&mut renderer, &mut processor, config, input);
    mato_dbg!("transformed in:\t\t{:?}", start.elapsed());
    output
}

#[cfg(test)]
mod tests {
    use mato::config::Config;

    fn matofmt(input: &str) -> String {
        super::matofmt(&Config::default(), input)
    }

    #[test]
    fn literal() {
        assert_eq!(matofmt("hallo"), "hallo");
    }
}
