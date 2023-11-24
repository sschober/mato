use std::path::{Path, PathBuf};

#[macro_export]
macro_rules! log_inf {
    ($config:ident, $( $args:expr ), *) => {
       if $config.log_level >= 1 {
           eprintln!( $( $args ),* );
       }
    };
}

#[macro_export]
macro_rules! log_dbg {
    ($config:ident, $( $args:expr ), *) => {
       if $config.log_level >= 2 {
           eprintln!( $( $args ),* );
       }
    };
}

#[macro_export]
macro_rules! log_trc {
    ($config:ident, $( $args:expr ), *) => {
       if $config.log_level >= 3 {
           eprintln!( $( $args ),* );
       }
    };
}

/// captures configuration parsed from command line arguments
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Config {
    /// source file that is to be processed
    pub source_file: String,
    pub target_file: String,
    /// parent directory of source file
    pub parent_dir: String,
    /// should watch mode be activated?
    pub watch: bool,
    /// dump intermediate representation (groff or latex)
    pub dump_groff: bool,
    pub dump_groff_file: bool,
    pub skip_rendering: bool,
    pub log_level: u8,
    /// language
    pub lang: String,
    pub preamble: String,
}
impl Config {
    pub const fn default() -> Self {
        Config {
            source_file: String::new(),
            target_file: String::new(),
            parent_dir: String::new(),
            watch: false,
            dump_groff: false,
            dump_groff_file: false,
            skip_rendering: false,
            log_level: 0,
            lang: String::new(),
            preamble: String::new(),
        }
    }
    /// create a configuration struct directly from `env::args.collect()`
    #[must_use]
    pub fn from(args: Vec<String>) -> Self {
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
                    "-v" | "--verbose" => result.log_level = 1,
                    "-d" | "--debug" => result.log_level = 2,
                    "-t" | "--trace" => result.log_level = 3,
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
            result.parent_dir = Path::new(&result.source_file)
                .parent()
                .expect("could not establish parent path of file")
                .as_os_str()
                .to_str()
                .unwrap()
                .to_string();
        }
        result
    }

    pub fn target_file(self: &Self, extentions: &str) -> PathBuf {
        let path_source_file = Path::new(&self.source_file);
        let mut path_target_file = path_source_file.to_path_buf();
        path_target_file.set_extension(extentions);
        path_target_file
    }

    pub fn set_target_file(self: &mut Self, extentions: &str) {
        self.target_file = self.target_file(extentions).to_str().unwrap().to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::Config;
    #[test]
    fn empty_args() {
        let config = Config::from(vec![]);
        assert_eq!(config.source_file, "");
    }
    #[test]
    fn many_args() {
        // test gets called in ./mato/ working directory
        let readme = format!(
            "{}/../README.md",
            std::env::current_dir().unwrap().to_str().unwrap()
        );
        let config = Config::from(vec![
            "-w".to_string(),
            "--dump-groff-file".to_string(),
            readme.to_string(),
        ]);
        assert_eq!(config.source_file, readme);
        assert!(config.watch);
        assert!(config.dump_groff_file);
    }
}
