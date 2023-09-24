pub mod canonicalize;
pub mod chain;
pub mod meta_data_extractor;
pub mod image_converter;
pub mod code_block;
pub mod identity;

use std::collections::HashMap;

use crate::{Exp, config::Config};

/// A processor processes the AST in some way
pub trait Process {
    fn process(& mut self, exp: Exp, config: &Config) -> Exp;
    fn get_context(& mut self) -> HashMap<String,String>; 
}