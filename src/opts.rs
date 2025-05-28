use std::collections::HashMap;
/// We do our option parsing ourselves,
/// NIH (not inventied here) syndrome.

#[derive(Clone, Debug)]
pub enum Opt {
    Flag {
        short_name: String,
        long_name: String,
        description: String,
    },
    Value {
        short_name: String,
        long_name: String,
        description: String,
        default: String,
    },
}

impl Opt {
    pub fn is_set(&self, r: &ParserResult) -> bool {
        match self {
            Opt::Flag {
                short_name: _,
                long_name,
                description: _,
            } => r.get_flag(long_name),
            _ => false,
        }
    }

    pub fn val(&self, r: &ParserResult) -> String {
        match self {
            Opt::Value {
                short_name: _,
                long_name,
                description: _,
                default: _,
            } => r.get_opt(long_name),
            _ => "".to_owned(),
        }
    }
}
/// Parser captures vectors of Opts and ValOpts
pub struct Parser {
    pub long_opts: HashMap<String, Opt>,
    pub short_opts: HashMap<String, Opt>,
}

pub fn print_version(prog: &str, version: &str) {
    println!("{} - {}", prog, version);
}

/// ParserResult captures the parsed entites extracted
/// from the command line arguments.
#[derive(Debug)]
pub struct ParserResult {
    pub command_name: String,
    /// Maps long name key to optional value: `key = value`.
    /// value is empty string if option was a flag.
    pub opts: HashMap<String, String>,
    /// arguments on the command line, that were not options
    pub params: Vec<String>,
    long_opts: HashMap<String, Opt>,
}

impl ParserResult {
    /// `true`, if the flag was given, `false` otherwise
    pub fn get_flag(&self, key: &str) -> bool {
        self.opts.contains_key(key)
    }

    /// gets option value. guaranteed to return a value, as
    /// defaults are part of the definition.
    pub fn get_opt(&self, key: &str) -> String {
        // TODO this is not so easy, because if a key is not
        // present in self.opts, we need to look it up in the
        // given opts of the parser, which we do not have here
        // in the results
        self.opts
            .get(key)
            .cloned()
            .unwrap_or_else(|| match self.long_opts.get(key).unwrap() {
                Opt::Flag {
                    short_name: _,
                    long_name: _,
                    description: _,
                } => "".to_owned(),
                Opt::Value {
                    short_name: _,
                    long_name: _,
                    description: _,
                    default,
                } => default.clone(),
            })
    }

    pub fn print_usage_string(&self) {
        // hashmap keys are not sorted, so we sort them
        // I find this a bit awkward. maybe there is a better way to do that.
        let mut sorted_keys = self.long_opts.keys().collect::<Vec<&String>>();
        // why does sort mutate and not return a sorted vec?
        sorted_keys.sort();
        for key in sorted_keys {
            let value = self.long_opts.get(key).unwrap();
            match value {
                Opt::Flag {
                    short_name,
                    long_name,
                    description,
                } => {
                    println!(
                        "\t-{0:<1}, --{1:<21}{2}",
                        short_name, long_name, description
                    )
                }
                Opt::Value {
                    short_name,
                    long_name,
                    description,
                    default,
                } => {
                    println!(
                        "\t-{0:<1} <val>, --{1:<15}{2}",
                        short_name,
                        format!("{} <val>", long_name),
                        format!("{} Default is value '{}'.", description, default)
                    )
                }
            }
        }
    }

    pub fn handle_standard_flags(&self, prog_name: &str, version: &str) {
        if self.opts.contains_key("version") {
            print_version(prog_name, version);
            std::process::exit(0);
        }
        if self.opts.contains_key("help") {
            print_version(prog_name, version);
            self.print_usage_string();
            std::process::exit(0);
        }
    }
}

#[macro_export]
macro_rules! opt_val {
    ($sn:tt, $ln:tt, $ds:tt, $de:tt) => {
        $crate::opts::Opt::Value {
            short_name: $sn.to_owned(),
            long_name: $ln.to_owned(),
            description: $ds.to_owned(),
            default: $de.to_owned(),
        }
    };
}

#[macro_export]
macro_rules! opt_flag {
    ($sn:tt, $ln:tt, $ds:tt) => {
        $crate::opts::Opt::Flag {
            short_name: $sn.to_owned(),
            long_name: $ln.to_owned(),
            description: $ds.to_owned(),
        }
    };
}

impl Parser {
    pub fn new() -> Self {
        let mut p = Parser {
            long_opts: HashMap::new(),
            short_opts: HashMap::new(),
        };
        // add standard flags for help and version
        p.add_opt(opt_flag!("h", "help", "Print command usage and exit"));
        p.add_opt(opt_flag!("v", "version", "Print command version and exit"));
        p.add_opt(opt_flag!("V", "verbose", "Enable verbose output."));
        p.add_opt(opt_flag!("d", "debug", "Enable debug output."));
        p.add_opt(opt_flag!("t", "trace", "Enable trace output."));
        p
    }

    /// registrs given option to this parser
    ///
    /// returns a references to the parser, so calls can be
    /// chained
    pub fn add_opt(&mut self, opt: Opt) -> Opt {
        match &opt {
            Opt::Flag {
                short_name,
                long_name,
                description: _,
            } => {
                self.long_opts.insert(long_name.clone(), opt.clone());
                self.short_opts.insert(short_name.clone(), opt.clone());
            }
            Opt::Value {
                short_name,
                long_name,
                description: _,
                default: _,
            } => {
                self.long_opts.insert(long_name.clone(), opt.clone());
                self.short_opts.insert(short_name.clone(), opt.clone());
            }
        }
        opt
    }

    /// inserts `key = ""`  (long_name) into `h` for flags, and
    /// `key = arg+1` for options.
    ///
    /// returns the index of element that was added (can be ignored then by caller)
    fn handle_match(
        &self,
        opt: &Opt,
        h: &mut HashMap<String, String>,
        pos: usize,
        args: &[String],
    ) -> usize {
        // different things need to be done, if opt is a
        // val or a flag
        match opt {
            Opt::Flag {
                short_name: _,
                long_name,
                description: _,
            } => {
                h.insert(long_name.clone(), "".to_string());
                0
            }
            Opt::Value {
                short_name: _,
                long_name,
                description: _,
                default: _,
            } => {
                if pos + 1 < args.len() {
                    h.insert(long_name.clone(), args[pos + 1].clone());
                    pos + 1
                } else {
                    panic!("option without value: {}", long_name)
                }
            }
        }
    }
    /// iterates over args and extracts known options,
    /// returns ParserResult containing all parsed options, and
    /// parameters, that were not options
    pub fn parse(&self, args: Vec<String>) -> ParserResult {
        let mut h = HashMap::new();
        let mut p: Vec<String> = Vec::new();
        let mut c = String::new();
        let mut skip_pos = 0;
        for (pos, arg) in args.iter().enumerate() {
            let opt_name = arg.trim_start_matches('-');
            if arg.starts_with("--") {
                if let Some(opt) = self.long_opts.get(opt_name) {
                    skip_pos = self.handle_match(opt, &mut h, pos, &args)
                }
            } else if arg.starts_with("-") {
                // splitting the option name enables
                // aggregated short opt clusters
                // like `-wv`; this even works, when the
                // last cluster option is a value option
                for opt_name in opt_name.split("") {
                    if let Some(opt) = self.short_opts.get(opt_name) {
                        skip_pos = self.handle_match(opt, &mut h, pos, &args)
                    }
                }
            } else {
                // eprintln!("pos: {}, skip_pos: {}", pos, skip_pos);
                if pos > 0 && pos != skip_pos {
                    // we skip positional parameter 0, as that is
                    // the command name
                    p.push(arg.clone())
                } else {
                    // save command name
                    c = arg.clone()
                }
            }
        }
        ParserResult {
            command_name: c,
            opts: h,
            params: p,
            long_opts: self.long_opts.clone(),
        }
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn add_opts() {
        let mut p = Parser::new();
        let opt = opt_flag!("s", "some", "Some option");
        assert_eq!(p.short_opts.len(), 5);
        p.add_opt(opt);
        assert_eq!(p.short_opts.len(), 6);
    }
    #[test]
    fn parse_version_opt() {
        let mut p = Parser::new();
        let opt = opt_flag!("s", "some", "Some option");
        p.add_opt(opt);
        let r = p.parse(vec!["-s".to_string()]);
        eprintln!("{:?}", r);
        assert_eq!(r.opts.get("some"), Some(&"".to_string()))
    }
    #[test]
    fn parse_source_file_val_opt() {
        let mut p = Parser::new();
        let opt = opt_val!("s", "source-file", "Some option", "def");
        p.add_opt(opt);
        let r = p.parse(vec!["-s".to_string(), "LICENSE".to_string()]);
        eprintln!("{:?}", r);
        assert_eq!(r.opts.get("source-file"), Some(&"LICENSE".to_string()))
    }

    #[test]
    fn parse_short_opt_cluster() {
        let p = Parser::new();
        let r = p.parse(vec!["-hv".to_string()]);
        eprintln!("{:?}", r);
        assert!(r.get_flag("version"), "version flag set");
        assert!(r.get_flag("help"), "help flag set");
    }

    #[test]
    fn parse_short_opt_cluster_with_val() {
        let mut p = Parser::new();
        let opt = opt_val!("l", "lang", "Set language", "den");
        p.add_opt(opt);
        let r = p.parse(vec!["-hl".to_string(), "en".to_string()]);
        eprintln!("{:?}", r);
        assert!(r.get_flag("help"), "help flag set");
        assert_eq!(r.get_opt("lang"), "en".to_string());
    }
}
