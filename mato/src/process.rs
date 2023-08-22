pub mod canonicalize;
pub mod chain;
pub mod meta_data_extractor;

use std::collections::HashMap;

use crate::Exp;

/// A processor processes the AST in some way
pub trait Process {
    fn process(& mut self, exp: Exp) -> Exp;
    fn get_context(& mut self) -> HashMap<String,String>; 
}