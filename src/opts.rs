use std::collections::HashMap;
/// We do our option parsing ourselves, NIH (not inventied here) syndrome.

/// Parser captures vectors of Opts and ValOpts
pub struct Parser {
    pub opts: Vec<Opt>,
    pub val_opts: Vec<ValOpt>,
}

/// An Opt has a short name, a lon name and a description, but no value.
#[derive(Debug)]
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
pub struct ValOpt {
    pub short_name: String,
    pub long_name: String,
    pub description: String,
    pub value: String,
}

impl Parser {
    pub fn new() -> Self {
        Parser {
            opts: vec![],
            val_opts: vec![],
        }
    }
    pub fn add_opt(&mut self, opt: Opt) -> &Self {
        self.opts.push(opt);
        self
    }

    /// iterates over args and extracts known options,
    /// returns a HashMap containing all parsed options
    pub fn parse(&self, args: Vec<String>) -> HashMap<String, String> {
        let mut h = HashMap::new();
        eprintln!("{:?}", self.opts);
        for arg in args {
            let arg = arg.trim_start_matches('-');
            self.opts
                .iter()
                .find(|o| o.short_name == arg || o.long_name == arg)
                .map(|o| h.insert(o.long_name.clone(), "".to_string()));
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
        assert_eq!(p.opts.len(), 0);
        p.add_opt(opt);
        assert_eq!(p.opts.len(), 1);
    }
    #[test]
    fn parse_version_opt() {
        let mut p = Parser {
            opts: vec![],
            val_opts: vec![],
        };
        let opt = Opt::new("v", "version", "Print version");
        p.add_opt(opt);
        let r = p.parse(vec!["-v".to_string()]);
        eprintln!("{:?}", r);
        assert_eq!(r.get(&"version".to_string()), Some(&"".to_string()))
    }
}
