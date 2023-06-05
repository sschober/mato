/// captures configuration parsed from command line arguments
#[derive(Debug, PartialEq)]
pub struct Config {
    /// source file that is to be processed
    pub source_file: String,
    /// should watch mode be activated?
    pub watch: bool,
    /// dump intermediate representation (groff or latex)
    pub dump: bool,
}

impl Config {
    /// create a configuration struct directly from env::Args
    pub fn from(args: Vec<String>) -> Config {
        let mut source_file = "".to_string();
        let mut watch = false;
        let mut dump: bool = false;
        for arg in args {
            match arg.as_str() {
                "-w" => watch = true,
                "-d" => dump = true,
                _ => source_file = arg,
            }
        }
        Config {
            source_file,
            watch,
            dump,
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
