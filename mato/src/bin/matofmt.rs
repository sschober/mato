use std::env;
use std::io;

use std::path::Path;
use std::time::Instant;

use mato::config::Config;
use mato::process::identity;
use mato::render::markdown;

fn main() -> std::io::Result<()> {
    let config = Config::from(env::args().collect());
    eprintln!("config: {:#?}", config);

    // open source file to be able watch it (we need a file descriptor)
    eprintln!("source file:\t\t{}", &config.source_file);
    if !config.source_file.is_empty() && !Path::new(&config.source_file).exists() {
        eprintln!("Could not open source file: {}", config.source_file);
        std::process::exit(1);
    }

    transform_and_render(&config, &config.source_file);
    Ok(())
}

fn matofmt(input: &str, config: &Config) -> String {
    let mut processor = identity::Identity {};
    mato::transform(
        &mut markdown::Renderer::new(),
        &mut processor,
        config,
        input,
    )
}

fn read_all_from_stdin() -> String {
    let lines = io::stdin().lines();
    let mut result = String::new();
    for line in lines {
        result.push_str(line.unwrap().as_str());
        result.push('\n');
    }
    result
}
fn transform_and_render(config: &Config, source_file: &str) {
    let start = Instant::now();
    let input = if source_file.is_empty() {
        read_all_from_stdin()
    } else {
        std::fs::read_to_string(source_file).unwrap()
    };
    eprintln!("read in:\t\t{:?}", start.elapsed());
    if config.dump {
        eprintln!("{}", input);
    }
    let start = Instant::now();
    let output = matofmt(&input, config);
    eprintln!("transformed in:\t\t{:?}", start.elapsed());

    println!("{output}");
}

#[cfg(test)]
mod tests {
    use mato::config::Config;

    use super::matofmt;

    fn matogro(input: &str) -> String {
        matofmt(input, &Config::new())
    }

    #[test]
    fn literal() {
        assert_eq!(matogro("hallo"), "hallo");
    }
}
