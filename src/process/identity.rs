use crate::log_trc;

use crate::Process;

/// Identity processor does nothing and just returns an
/// identical AST
#[derive(Debug)]
pub struct Identity {}

impl Process for Identity {
    fn process(
        &mut self,
        exp: crate::syntax::Tree,
        config: &crate::config::Config,
    ) -> crate::syntax::Tree {
        log_trc!(config, "{:?}", self);
        exp
    }
}
pub fn new() -> Box<dyn Process> {
    Box::new(Identity {})
}
