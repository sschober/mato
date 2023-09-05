use std::collections::HashMap;

use crate::{syntax::{Exp, meta_data_block, meta_data_item}, config::Config};

use super::Process;

pub struct MetaDataExtractor {
    ctx: HashMap<String, String>,
}

impl MetaDataExtractor {
    pub fn new() -> Self {
        Self {
            ctx: HashMap::new(),
        }
    }

    pub fn from(custom_preamble: &str) -> Self {
        let mut map = HashMap::new();
        map.insert("custom_preamble".to_string(), custom_preamble.to_string());
        Self {
            ctx: map,
        }
    }

    fn extract_meta_data(&mut self, exp: Exp) -> Exp {
        match exp {
            Exp::Cat(b1, b2) => self.extract_meta_data(*b1).cat(self.extract_meta_data(*b2)),
            Exp::MetaDataBlock(e) => meta_data_block(self.extract_meta_data(*e)),
            Exp::MetaDataItem(k, v) => {
                eprintln!("inserting {} = {}", k, v);
                self.ctx.insert(k.to_string(), v.to_string());
                meta_data_item(k,v)
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
    fn process(&mut self, exp: Exp, _: &Config) -> crate::syntax::Exp {
        self.extract_meta_data(exp)
    }

    fn get_context(&mut self) -> HashMap<String, String> {
        self.ctx.clone()
    }
}
