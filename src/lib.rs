//! markdown transformer toolkit

use parser::Parser;
use syntax::Exp;
pub mod parser;
pub mod renderer;
pub mod watch;
pub mod syntax;

/// top-level helper method to transform a given input string into a target language specified by the passed in renderer
pub fn transform<T: renderer::Renderer>(t: T, input: &str) -> String {
    let result = Parser::parse(input);
    render(t, result)
}

fn render<T: renderer::Renderer>(t: T, exp: Exp) -> String {
    t.render(exp)
}
