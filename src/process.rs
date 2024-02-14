pub mod canonicalize;
pub mod chain;
pub mod code_block;
pub mod identity;
pub mod image_converter;
pub mod meta_data_extractor;

use core::fmt::Debug;
use std::collections::HashMap;

use crate::{config::Config, Tree};

/// A processor processes the AST in some way
pub trait Process : Debug {
    fn process(&mut self, exp: Tree, config: &Config) -> Tree;
    fn get_context(&mut self) -> HashMap<String, String>;
}