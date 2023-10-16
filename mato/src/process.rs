pub mod canonicalize;
pub mod chain;
pub mod code_block;
pub mod identity;
pub mod image_converter;
pub mod meta_data_extractor;

use std::collections::HashMap;

use crate::{config::Config, Exp};

/// A processor processes the AST in some way
pub trait Process {
    fn process(&mut self, exp: Exp, config: &Config) -> Exp;
    fn get_context(&mut self) -> HashMap<String, String>;
}
