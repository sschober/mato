//! components related to rendering a syntax tree to a concrete target language, like groff or latex
pub mod groff;
pub mod markdown;

use std::collections::HashMap;

use crate::Exp;

/// A renderer renders an Exp into a String
pub trait Render {
    /// render the passed-in expression into a string
    fn render(&mut self, exp: Exp, ctx: HashMap<String, String>) -> String;
}
