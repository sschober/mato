//! markdown transformer toolkit

use parser::Parser;
use syntax::Exp;
pub mod config;
pub mod parser;
pub mod render;
pub mod syntax;
pub mod watch;

/// top-level helper method to transform a given input string into a target language specified by the passed in renderer
pub fn transform<T: render::Render>(t: & mut T, input: &str) -> String {
    let result = Parser::parse(input);
    render(t, result)
}

/// helper function for static dispatch
///
/// calls the passed in renderer on the result created by the parser
fn render<T: render::Render>(t: & mut T, exp: Exp) -> String {
    t.render(exp)
}
