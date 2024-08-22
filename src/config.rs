use std::{
    fs,
    path::{Path, PathBuf},
};

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
    pub skip_preamble: bool,
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
                .ok_or(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "could not establish parent of source file",
                ))?
                .as_os_str()
                .to_str()
                .unwrap()
                .to_string();
        }
        Ok(result)
    }

    pub fn target_file(&self, extention: &str) -> PathBuf {
        crate::replace_file_extension(&self.source_file, extention)
    }

    pub fn set_target_file(&mut self, extentions: &str) {
        self.target_file = self.target_file(extentions).to_str().unwrap().to_string();
    }

    pub fn locate_and_load_preamble(&mut self, name: &str, default: &str) {
        let sibbling_preamble = Path::new(&self.parent_dir).join(name);
        if sibbling_preamble.as_path().is_file() {
            log_dbg!(
                self,
                "found sibbling preamble: {}",
                sibbling_preamble.display()
            );
            self.preamble = fs::read_to_string(sibbling_preamble).unwrap();
        } else {
            self.preamble = default.to_string();
            log_dbg!(self, "preamble:\t\tbuilt-in");
        }
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
