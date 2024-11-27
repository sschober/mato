use std::path::Path;

/// captures configuration parsed from command line arguments
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Config {
    /// source file that is to be processed
    pub source_file: String,
    /// should watch mode be activated?
    pub watch: bool,
    /// dump intermediate representation
    pub dump_groff: bool,
    pub dump_groff_file: bool,
    pub skip_rendering: bool,
    /// language
    pub lang: String,
    pub skip_preamble: bool,
}

impl Config {
    pub const fn default() -> Self {
        Config {
            source_file: String::new(),
            watch: false,
            dump_groff: false,
            dump_groff_file: false,
            skip_rendering: false,
            lang: String::new(),
            skip_preamble: false,
        }
    }

    /// create a configuration struct directly from `env::args.collect()`
    pub fn from(args: Vec<String>) -> Result<Config, std::io::Error> {
        let mut result = Self::default();
        result.lang = "den".to_string();
        if args.len() > 1 {
            for arg in args {
                match arg.as_str() {
                    "-w" => result.watch = true,
                    "--dump-groff-file" => result.dump_groff_file = true,
                    "-Z" => {
                        result.skip_rendering = true;
                        result.dump_groff = true;
                    }
                    "-len" | "-l en" => result.lang = "en".to_string(),
                    "-" => result.source_file = String::new(),
                    _ => result.source_file = arg,
                }
            }
        }
        if !result.source_file.is_empty() {
            if !Path::new(&result.source_file).exists() {
                eprintln!("Could not open source file: {}", result.source_file);
                std::process::exit(1);
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::Config;
    #[test]
    fn empty_args() {
        let config = Config::from(vec![]).unwrap();
        assert_eq!(config.source_file, "");
    }
    #[test]
    fn many_args() {
        // test gets called in ./mato/ working directory
        let readme = format!(
            "{}/README.md",
            std::env::current_dir().unwrap().to_str().unwrap()
        );
        let config = Config::from(vec![
            "-w".to_string(),
            "--dump-groff-file".to_string(),
            readme.to_string(),
        ])
        .unwrap();
        assert_eq!(config.source_file, readme);
        assert!(config.watch);
        assert!(config.dump_groff_file);
    }
}
