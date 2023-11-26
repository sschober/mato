pub mod canonicalize;
pub mod chain;
pub mod code_block;
pub mod identity;
pub mod image_converter;
pub mod meta_data_extractor;

use core::fmt::Debug;
use std::collections::HashMap;

use crate::{config::Config, Exp};

/// A processor processes the AST in some way
pub trait Process {
    fn process(&mut self, exp: Exp, config: &Config) -> Exp;
    fn get_context(&mut self) -> HashMap<String, String>;
    fn get_name(&self) -> String;
}

impl Debug for dyn Process {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_name())
    }
}
