use crate::m_trc;
use crate::Process;

/// Identity processor does nothing and just returns an
/// identical AST
#[derive(Debug)]
pub struct Identity {}

impl Process for Identity {
    fn process(
        &mut self,
        exp: crate::syntax::Tree,
        _config: &crate::config::Config,
    ) -> crate::syntax::Tree {
        m_trc!("{:?}", self);
        exp
    }
}
pub fn new() -> Box<dyn Process> {
    Box::new(Identity {})
}
