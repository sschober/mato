//! markdown transformer toolkit

use parser::Parser;
use syntax::Exp;
pub mod parser;
pub mod renderer;
pub mod watch;
pub mod syntax;
pub mod config;

/// top-level helper method to transform a given input string into a target language specified by the passed in renderer
pub fn transform<T: renderer::Renderer>(t: T, input: &str) -> String {
    let result = Parser::parse(input);
    render(t, result)
}

/// helper function for static dispatch
///
/// calls the passed in renderer on the result created by the parser
fn render<T: renderer::Renderer>(t: T, exp: Exp) -> String {
    t.render(exp)
}
