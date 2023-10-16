use std::env;

use mato::config::Config;
use mato::process::canonicalize;
use mato::render::tex::Renderer;

fn main() {
    for file in env::args().skip(1) {
        let input = std::fs::read_to_string(file).unwrap();
        println!(
            "{}",
            mato::transform(
                &mut Renderer {},
                &mut canonicalize::Canonicalizer {},
                &Config::new(),
                input.as_str()
            )
        );
    }
}

#[cfg(test)]
mod tests {
    use mato::config::Config;
    use mato::process::canonicalize;
    use mato::render::tex::Renderer;

    #[test]
    fn literal() {
        assert_eq!(
            mato::transform(
                &mut Renderer {},
                &mut canonicalize::Canonicalizer {},
                &Config::new(),
                "hallo"
            ),
            "hallo"
        );
    }
    #[test]
    fn italic() {
        assert_eq!(
            mato::transform(
                &mut Renderer {},
                &mut canonicalize::Canonicalizer {},
                &Config::new(),
                "_hallo_"
            ),
            "\\textit{hallo}"
        );
    }
    #[test]
    fn bold() {
        assert_eq!(
            mato::transform(
                &mut Renderer {},
                &mut canonicalize::Canonicalizer {},
                &Config::new(),
                "*hallo*"
            ),
            "\\textbf{hallo}"
        );
    }
}
