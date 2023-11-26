use std::collections::HashMap;

use crate::log_trc;

use super::Process;

/// Identity processor does nothing and just returns an
/// identical AST
pub struct Identity {}

impl Process for Identity {
    fn process(
        &mut self,
        exp: crate::syntax::Exp,
        config: &crate::config::Config,
    ) -> crate::syntax::Exp {
        log_trc!(config, "{}", self.get_name());
        exp
    }

    fn get_context(&mut self) -> std::collections::HashMap<String, String> {
        HashMap::new()
    }

    fn get_name(&self) -> String {
        "Identity".to_string()
    }
}
pub fn new() -> Box<dyn Process> {
    Box::new(Identity {})
}
