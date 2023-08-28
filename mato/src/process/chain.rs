use std::collections::HashMap;

use super::Process;

use crate::{Exp, config::Config};

pub struct Chain {
    pub a: Box<dyn Process>,
    pub b: Box<dyn Process>
}

impl Process for Chain {
    fn process(&mut self, exp: Exp, config: &Config) -> Exp {
        let result = self.a.process(exp, config);
        self.b.process(result, config)
    }

    fn get_context(& mut self) -> HashMap<String,String> {
        let result = self.a.get_context();
        result.into_iter().chain(self.b.get_context()).collect()
    }
}

