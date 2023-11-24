use std::env;

use std::path::Path;
use std::time::Instant;

use mato::config::Config;
use mato::process::identity;
use mato::render::markdown;

fn main() -> std::io::Result<()> {
    let config = Config::from(env::args().collect())?;
    eprintln!("config: {:#?}", config);

    // open source file to be able watch it (we need a file descriptor)
    eprintln!("source file:\t\t{}", &config.source_file);
    if !config.source_file.is_empty() && !Path::new(&config.source_file).exists() {
        eprintln!("Could not open source file: {}", config.source_file);
        std::process::exit(1);
    }
    let input = mato::read_input(&config);
    println!("{}", matofmt(&config, &input));
    Ok(())
}

fn matofmt(config: &Config, input: &str) -> String {
    let start = Instant::now();
    let mut processor = identity::Identity {};
    let mut renderer = markdown::Renderer::new();
    let output = mato::transform(&mut renderer, &mut processor, config, input);
    eprintln!("transformed in:\t\t{:?}", start.elapsed());
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
