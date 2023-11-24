use std::collections::HashMap;

use super::Process;

use crate::{config::Config, Exp};

/// A Chain can be used to chain multiple processors
/// together and form a transformation chain or pipeline.
pub struct Chain {
    pub a: Box<dyn Process>,
    pub b: Box<dyn Process>,
}

impl Process for Chain {
    fn process(&mut self, exp: Exp, config: &Config) -> Exp {
        let result = self.a.process(exp, config);
        self.b.process(result, config)
    }

    fn get_context(&mut self) -> HashMap<String, String> {
        let result = self.a.get_context();
        result.into_iter().chain(self.b.get_context()).collect()
    }
}

impl Chain {
    pub fn append(self, p: Box<dyn Process>) -> Self {
        new(Box::new(self), p)
    }
}

pub fn new(a: Box<dyn Process>, b: Box<dyn Process>) -> Chain {
    Chain { a, b }
}
