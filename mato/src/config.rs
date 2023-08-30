use std::path::Path;

/// captures configuration parsed from command line arguments
#[derive(Debug, PartialEq, Eq, Default)]
pub struct Config {
    /// source file that is to be processed
    pub source_file: String,
    /// parent directory of source file
    pub parent_dir: String,
    /// should watch mode be activated?
    pub watch: bool,
    /// dump intermediate representation (groff or latex)
    pub dump: bool,
    /// language
    pub lang: String,
}

impl Config {
    pub fn new() -> Self {
        Default::default()
    }
    /// create a configuration struct directly from `env::args.collect()`
    #[must_use]
    pub fn from(args: Vec<String>) -> Self {
        let mut source_file = String::new();
        let mut watch = false;
        let mut dump: bool = false;
        let mut lang: String = "den".to_string();
        for arg in args {
            match arg.as_str() {
                "-w" => watch = true,
                "-d" => dump = true,
                "-len" | "-l en" => lang = "en".to_string(),
                _ => source_file = arg,
            }
        }
        let path_source_file = Path::new(&source_file);
        let mut parent_dir = String::new();
        if ! source_file.is_empty() {
            parent_dir = path_source_file
                .parent()
                .expect("could not establish parent path of file")
                .as_os_str()
                .to_str()
                .unwrap()
                .to_string();
        }
        Self {
            source_file,
            parent_dir,
            watch,
            dump,
            lang,
        }
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
        let config = Config::from(vec![
            "-w".to_string(),
            "-d".to_string(),
            "source_file".to_string(),
        ]);
        assert_eq!(config.source_file, "source_file");
        assert!(config.watch);
        assert!(config.dump);
    }
}
