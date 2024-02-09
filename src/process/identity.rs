use std::collections::HashMap;

use crate::log_trc;

use super::Process;

/// Identity processor does nothing and just returns an
/// identical AST
#[derive(Debug)]
pub struct Identity {}

impl Process for Identity {
    fn process(
        &mut self,
        exp: crate::syntax::Exp,
        config: &crate::config::Config,
    ) -> crate::syntax::Exp {
        log_trc!(config, "{:?}", self);
        exp
    }

    fn get_context(&mut self) -> std::collections::HashMap<String, String> {
        HashMap::new()
    }
}
pub fn new() -> Box<dyn Process> {
    Box::new(Identity {})
}
