use std::collections::{BTreeSet, HashMap};
/// We do our option parsing ourselves, NIH (not inventied here) syndrome.

/// Parser captures vectors of Opts and ValOpts
pub struct Parser {
    pub long_opts: HashMap<String, Opt>,
    pub long_val_opts: HashMap<String, ValOpt>,
    pub short_opts: HashMap<String, Opt>,
    pub short_val_opts: HashMap<String, ValOpt>,
    pub ordered_long_opt_keys: BTreeSet<String>,
    pub ordered_short_opt_keys: BTreeSet<String>,
}

/// An Opt has a short name, a lon name and a description, but no value.
#[derive(Debug, Clone)]
pub struct Opt {
    pub short_name: String,
    pub long_name: String,
    pub description: String,
}

impl Opt {
    pub fn new(s: &str, l: &str, d: &str) -> Self {
        Opt {
            short_name: s.to_string(),
            long_name: l.to_string(),
            description: d.to_string(),
        }
    }
}

/// A ValOpt has a short name, a long name, a description and, opposed to Val, a value.
#[derive(Debug, Clone)]
pub struct ValOpt {
    pub short_name: String,
    pub long_name: String,
    pub description: String,
    pub value: String,
}

impl ValOpt {
    pub fn new(short_name: &str, long_name: &str, description: &str, value: &str) -> Self {
        Self {
            short_name: short_name.to_string(),
            long_name: long_name.to_string(),
            description: description.to_string(),
            value: value.to_string(),
        }
    }
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            long_opts: HashMap::new(),
            long_val_opts: HashMap::new(),
            short_opts: HashMap::new(),
            short_val_opts: HashMap::new(),
            ordered_long_opt_keys: BTreeSet::new(),
            ordered_short_opt_keys: BTreeSet::new(),
        }
    }
    pub fn add_opt(&mut self, opt: Opt) -> &Self {
        self.ordered_long_opt_keys.insert(opt.long_name.clone());
        self.ordered_short_opt_keys.insert(opt.short_name.clone());
        self.long_opts.insert(opt.long_name.clone(), opt.clone());
        self.short_opts.insert(opt.short_name.clone(), opt);
        self
    }
    pub fn add_val_opt(&mut self, opt: ValOpt) -> &Self {
        self.ordered_long_opt_keys.insert(opt.long_name.clone());
        self.ordered_short_opt_keys.insert(opt.short_name.clone());
        self.long_val_opts
            .insert(opt.long_name.clone(), opt.clone());
        self.short_val_opts.insert(opt.short_name.clone(), opt);
        self
    }

    /// iterates over args and extracts known options,
    /// returns a HashMap containing all parsed options
    pub fn parse(&self, args: Vec<String>) -> HashMap<String, String> {
        let mut h = HashMap::new();
        eprintln!("long opt keys: {:?}", self.ordered_long_opt_keys);
        eprintln!("long opts: {:?}", self.long_opts);
        eprintln!("long val opts: {:?}", self.long_val_opts);
        eprintln!("short opt keys {:?}", self.ordered_short_opt_keys);
        for (pos, arg) in args.iter().enumerate() {
            let opt_name = arg.trim_start_matches('-');
            eprintln!("opt name: {}", opt_name);
            if arg.starts_with("--") {
                eprintln!("is long opt...");
                self.ordered_long_opt_keys
                    .iter()
                    .find(|o| opt_name.eq(*o))
                    .map(|o| {
                        if self.long_opts.get(o).is_some() {
                            eprintln!("found long opt: {}", o);
                            let opt = self.long_opts.get(o).unwrap();
                            h.insert(opt.long_name.clone(), "".to_string())
                        } else if self.long_val_opts.get(o).is_some() {
                            // we have a value option
                            let val_opt = self.long_val_opts.get(o).unwrap();
                            if pos + 1 < args.len() {
                                h.insert(val_opt.long_name.clone(), args[pos + 1].to_string())
                            } else {
                                panic!("Value option without value: --{}", val_opt.long_name)
                            }
                        } else {
                            None
                        }
                    });
            } else if arg.starts_with('-') {
                eprintln!("is short opt...");
                self.ordered_short_opt_keys
                    .iter()
                    .find(|o| opt_name.eq(*o))
                    .map(|o| {
                        if self.short_opts.get(o).is_some() {
                            eprintln!("found short opt: {}", o);
                            let opt = self.short_opts.get(o).unwrap();
                            h.insert(opt.long_name.clone(), "".to_string())
                        } else if self.short_val_opts.get(o).is_some() {
                            // we have a value option
                            eprintln!("found short val_opt: {}", o);
                            let val_opt = self.short_val_opts.get(o).unwrap();
                            if pos + 1 < args.len() {
                                h.insert(val_opt.long_name.clone(), args[pos + 1].to_string())
                            } else {
                                panic!("Value option without value: --{}", val_opt.long_name)
                            }
                        } else {
                            None
                        }
                    });
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
        let opt = Opt::new("v", "version", "Print version");
        assert_eq!(p.short_opts.len(), 0);
        p.add_opt(opt);
        assert_eq!(p.short_opts.len(), 1);
    }
    #[test]
    fn parse_version_opt() {
        let mut p = Parser::new();
        let opt = Opt::new("v", "version", "Print version");
        p.add_opt(opt);
        let r = p.parse(vec!["-v".to_string()]);
        eprintln!("{:?}", r);
        assert_eq!(r.get(&"version".to_string()), Some(&"".to_string()))
    }
    #[test]
    fn parse_source_file_val_opt() {
        let mut p = Parser::new();
        let opt = ValOpt::new("s", "source-file", "Print version", "");
        p.add_val_opt(opt);
        let r = p.parse(vec!["-s".to_string(), "LICENSE".to_string()]);
        eprintln!("{:?}", r);
        assert_eq!(
            r.get(&"source-file".to_string()),
            Some(&"LICENSE".to_string())
        )
    }
}
