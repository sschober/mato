//! components related to rendering a syntax tree to a concrete target language, like groff or latex
pub mod groff;
pub mod tex;

use crate::Exp;

/// A renderer renders an Exp into a String
pub trait Renderer {
    /// render the passed-in expression into a string
    fn render(&self, exp: Exp) -> String;
}
