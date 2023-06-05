use std::env;

use mato::render::tex::Renderer;

fn main() {
    for file in env::args().skip(1) {
        let input = std::fs::read_to_string(file).unwrap();
        println!("{}", mato::transform(&Renderer {}, input.as_str()));
    }
}

#[cfg(test)]
mod tests {
    use mato::render::tex::Renderer;

    #[test]
    fn literal() {
        assert_eq!(mato::transform(&Renderer {}, "hallo"), "hallo");
    }
    #[test]
    fn italic() {
        assert_eq!(mato::transform(&Renderer {}, "_hallo_"), "\\textit{hallo}");
    }
    #[test]
    fn bold() {
        assert_eq!(mato::transform(&Renderer {}, "*hallo*"), "\\textbf{hallo}");
    }
}
