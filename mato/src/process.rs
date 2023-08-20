pub mod canonicalize;

use crate::Exp;

/// A processor processes the AST in some way
pub trait Process {
    fn process(& mut self, exp: Exp) -> Exp;
}