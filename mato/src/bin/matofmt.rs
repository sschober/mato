use std::env;

use std::fs;
use std::path::Path;
use std::time::Instant;

use mato::config::Config;
use mato::process::identity;
use mato::render::markdown;


fn main() -> std::io::Result<()> {
    let config = Config::from(env::args().collect());
    eprintln!("config: {:#?}", config);

    // open source file to be able watch it (we need a file descriptor)
    println!("source file:\t\t{}", &config.source_file);
    if !Path::new(&config.source_file).exists() {
        eprintln!("Could not open source file: {}", config.source_file);
        std::process::exit(1);
    }
    let path_source_file = Path::new(&config.source_file);
    let mut path_target_file = path_source_file.to_path_buf();
    path_target_file.set_extension("fmt");
    println!("target file name:\t{}", path_target_file.display());

    transform_and_render(
        &config,
        &config.source_file,
        path_target_file.to_str().unwrap(),
    );
    Ok(())
}

fn matofmt(input: &str, config: &Config) -> String {
    let mut processor = identity::Identity{};
    mato::transform(&mut markdown::Renderer::new(), &mut processor, config, input)
}


fn transform_and_render(config: &Config, source_file: &str, target_file: &str) {
    let start = Instant::now();
    let input = std::fs::read_to_string(source_file).unwrap();
    println!("read in:\t\t{:?}", start.elapsed());

    let start = Instant::now();
    let output = matofmt(&input, config);
    println!("transformed in:\t\t{:?}", start.elapsed());
    if config.dump {
        println!("{output}");
    }

    let start = Instant::now();
    fs::write(target_file, output).expect("Unable to write out file");
    println!("written in:\t\t{:?} ", start.elapsed());
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
