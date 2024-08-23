use std::collections::HashMap;
/// We do our option parsing ourselves, NIH (not inventied here) syndrome.

#[derive(Clone)]
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
    },
}

/// Parser captures vectors of Opts and ValOpts
pub struct Parser {
    pub long_opts: HashMap<String, Opt>,
    pub short_opts: HashMap<String, Opt>,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            long_opts: HashMap::new(),
            short_opts: HashMap::new(),
        }
    }
    pub fn add_opt(&mut self, opt: Opt) -> &Self {
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
            } => {
                self.long_opts.insert(long_name.clone(), opt.clone());
                self.short_opts.insert(short_name.clone(), opt.clone());
            }
        }
        self
    }

    fn handle_match(
        &self,
        opt: &Opt,
        h: &mut HashMap<String, String>,
        pos: usize,
        args: &Vec<String>,
    ) {
        match opt {
            Opt::Flag {
                short_name: _,
                long_name,
                description: _,
            } => h.insert(long_name.clone(), "".to_string()),
            Opt::Value {
                short_name: _,
                long_name,
                description: _,
            } => {
                if pos + 1 < args.len() {
                    h.insert(long_name.clone(), args[pos + 1].clone())
                } else {
                    panic!("option without value: {}", long_name)
                }
            }
        };
    }
    /// iterates over args and extracts known options,
    /// returns a HashMap containing all parsed options
    pub fn parse(&self, args: Vec<String>) -> HashMap<String, String> {
        let mut h = HashMap::new();
        for (pos, arg) in args.iter().enumerate() {
            let opt_name = arg.trim_start_matches('-');
            eprintln!("opt name: {}", opt_name);
            if arg.starts_with("--") {
                match self.long_opts.get(opt_name) {
                    Some(opt) => self.handle_match(opt, &mut h, pos, &args),
                    None => {}
                }
            } else if arg.starts_with("-") {
                match self.short_opts.get(opt_name) {
                    Some(opt) => self.handle_match(opt, &mut h, pos, &args),
                    None => {}
                }
            }
        }
        h
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
        let opt = Opt::Flag {
            short_name: "v".to_string(),
            long_name: "version".to_string(),
            description: "Print version".to_string(),
        };
        assert_eq!(p.short_opts.len(), 0);
        p.add_opt(opt);
        assert_eq!(p.short_opts.len(), 1);
    }
    #[test]
    fn parse_version_opt() {
        let mut p = Parser::new();
        let opt = Opt::Flag {
            short_name: "v".to_string(),
            long_name: "version".to_string(),
            description: "Print version".to_string(),
        };
        p.add_opt(opt);
        let r = p.parse(vec!["-v".to_string()]);
        eprintln!("{:?}", r);
        assert_eq!(r.get(&"version".to_string()), Some(&"".to_string()))
    }
    #[test]
    fn parse_source_file_val_opt() {
        let mut p = Parser::new();
        let opt = Opt::Value {
            short_name: "s".to_string(),
            long_name: "source-file".to_string(),
            description: "Print version".to_string(),
        };
        p.add_opt(opt);
        let r = p.parse(vec!["-s".to_string(), "LICENSE".to_string()]);
        eprintln!("{:?}", r);
        assert_eq!(
            r.get(&"source-file".to_string()),
            Some(&"LICENSE".to_string())
        )
    }
}
