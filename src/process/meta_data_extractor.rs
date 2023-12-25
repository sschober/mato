use std::collections::HashMap;

use crate::{
    config::Config,
    log_trc,
    syntax::{meta_data_block, meta_data_item, Exp},
};

use super::Process;

/// The MetaDataExtractor takes the meta data header
/// block and fills a context object, which can
/// later during rendering be used to govern stylistic
/// decisions.
pub struct MetaDataExtractor {
    ctx: HashMap<String, String>,
}

impl MetaDataExtractor {
    pub fn new() -> Self {
        Self {
            ctx: HashMap::new(),
        }
    }

    fn extract_meta_data(&mut self, exp: Exp) -> Exp {
        match exp {
            Exp::Cat(b1, b2) => self.extract_meta_data(*b1).cat(self.extract_meta_data(*b2)),
            Exp::MetaDataBlock(e) => meta_data_block(self.extract_meta_data(*e)),
            Exp::MetaDataItem(k, v) => {
                self.ctx.insert(k.to_string(), v.to_string());
                meta_data_item(k, v)
            }
            _ => exp,
        }
    }
}

impl Default for MetaDataExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl Process for MetaDataExtractor {
    fn process(&mut self, exp: Exp, config: &Config) -> crate::syntax::Exp {
        log_trc!(config, "{}", self.get_name());
        self.extract_meta_data(exp)
    }

    fn get_context(&mut self) -> HashMap<String, String> {
        self.ctx.clone()
    }

    fn get_name(&self) -> String {
        "MetaDataExtractor".to_string()
    }
}

pub fn new(preamble: &str) -> Box<dyn Process> {
    let mut map = HashMap::new();
    let cp = preamble.to_string();
    if !cp.is_empty() {
        map.insert("preamble".to_string(), preamble.to_string());
    }
    Box::new(MetaDataExtractor { ctx: map })
}