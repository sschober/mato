//! markdown transformer toolkit

use config::Config;
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
    config: &Config,
    input: &str,
) -> String {
    let mut exp = Parser::parse(input);
    exp = process(p, exp, config);
    render(r, exp, p)
}

/// helper function for static dispatch
///
/// calls the passed in processor on the given exp
fn process<P: process::Process>(p: &mut P, exp: Exp, config: &Config) -> Exp {
    p.process(exp, config)
}

/// helper function for static dispatch
///
/// calls the passed in renderer on the result created by the parser
fn render<R: render::Render, P: process::Process>(r: &mut R, exp: Exp, p: &mut P) -> String {
    r.render(exp, p.get_context())
}
