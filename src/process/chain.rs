use std::time::Instant;

use crate::log::get_log_level;
use crate::{m_trc, Process};

use crate::Tree;
use core::fmt::Debug;

/// A Chain can be used to chain multiple processors
/// together and form a transformation chain or pipeline.
pub struct Chain<'a> {
    pub a: Box<dyn Process + 'a>,
    pub b: Box<dyn Process + 'a>,
}

impl Process for Chain<'_> {
    fn process(&mut self, exp: Tree) -> Tree {
        let start = Instant::now();
        let result = self.a.process(exp);
        if get_log_level() >= 2 {
            m_trc!("{:?}: {:?}", self.a, start.elapsed());
        }
        let result = self.b.process(result);
        if get_log_level() >= 2 {
            m_trc!("{:?}: {:?}", self.b, start.elapsed());
        }
        result
    }
}

impl Debug for Chain<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} -> {:?}", self.a, self.b)
    }
}

impl<'a> Chain<'a> {
    pub fn append(self, p: Box<dyn Process + 'a>) -> Self {
        new(Box::new(self), p)
    }
}

pub fn new<'a>(a: Box<dyn Process + 'a>, b: Box<dyn Process + 'a>) -> Chain<'a> {
    Chain { a, b }
}
