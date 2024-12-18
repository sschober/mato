use std::time::Instant;

use crate::log::get_log_level;
use crate::{m_trc, Process};

use crate::{config::Config, Tree};
use core::fmt::Debug;

/// A Chain can be used to chain multiple processors
/// together and form a transformation chain or pipeline.
pub struct Chain {
    pub a: Box<dyn Process>,
    pub b: Box<dyn Process>,
}

impl Process for Chain {
    fn process(&mut self, exp: Tree, config: &Config) -> Tree {
        let start = Instant::now();
        let result = self.a.process(exp, config);
        if get_log_level() >= 2 {
            m_trc!("{:?}: {:?}", self.a, start.elapsed());
        }
        let result = self.b.process(result, config);
        if get_log_level() >= 2 {
            m_trc!("{:?}: {:?}", self.b, start.elapsed());
        }
        result
    }
}

impl Debug for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} -> {:?}", self.a, self.b)
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
