use std::collections::HashMap;

use super::Process;

use crate::Exp;

pub struct Chain {
    pub a: Box<dyn Process>,
    pub b: Box<dyn Process>
}

impl Process for Chain {
    fn process(&mut self, exp: Exp) -> Exp {
        let result = self.a.process(exp);
        self.b.process(result)
    }

    fn get_context(& mut self) -> HashMap<String,String> {
        let result = self.a.get_context();
        result.into_iter().chain(self.b.get_context()).collect()
    }
}