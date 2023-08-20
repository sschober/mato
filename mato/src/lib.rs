//! markdown transformer toolkit

use parser::Parser;
use syntax::Exp;
pub mod config;
pub mod parser;
pub mod process;
pub mod render;
pub mod syntax;
pub mod watch;

/// top-level helper method to transform a given input string into a target language specified by the passed in renderer
pub fn transform<R: render::Render, P: process::Process>(
    r: &mut R,
    p: &mut P,
    input: &str,
) -> String {
    let mut exp = Parser::parse(input);
    exp = process(p, exp);
    render(r, exp)
}

/// helper function for static dispatch
/// 
/// calls the passed in processor on the given exp
fn process<P: process::Process>(p: &mut P, exp: Exp) -> Exp {
    p.process(exp)
}

/// helper function for static dispatch
///
/// calls the passed in renderer on the result created by the parser
fn render<T: render::Render>(t: &mut T, exp: Exp) -> String {
    t.render(exp)
}
