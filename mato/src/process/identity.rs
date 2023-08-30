use std::collections::HashMap;

use super::Process;

pub struct Identity {}

impl Process for Identity {
    fn process(& mut self, exp: crate::syntax::Exp, _: &crate::config::Config) -> crate::syntax::Exp {
        exp
    }

    fn get_context(& mut self) -> std::collections::HashMap<String,String> {
        HashMap::new()
    }
}