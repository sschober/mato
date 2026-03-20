use crate::{
    config::Config,
    m_dbg, m_trc,
    syntax::{image, lit, Tree},
};

use crate::Process;

/// ImageConverter processor currently only transforms
/// path information in the image expression.
/// Ultimately, we want it to do conversion and caching.
#[derive(Debug)]
pub struct ImageConverter<'a> {
    config: &'a Config,
}

impl ImageConverter<'_> {
    /// walks the tree until it find Tree::Image nodes, uses recursive descent
    fn walk(&mut self, exp: Tree) -> Tree {
        match exp {
            Tree::Document(dt, be) => Tree::Document(dt, Box::new(self.walk(*be))),
            Tree::Cat(b1, b2) => self.walk(*b1).cat(self.walk(*b2)),
            Tree::Image(caption, path, size_spec) => self.resolve_path(*caption, *path, *size_spec),
            _ => exp,
        }
    }

    /// resolves path specs in image references using the source file making them absolute
    fn resolve_path(&mut self, caption: Tree, path: Tree, size_spec: Tree) -> Tree {
        let path = match path {
            Tree::Literal(p) => {
                let mut resolved_path = p.clone();
                if !p.starts_with('/') {
                    let parent_dir_path = crate::parent_dir(&self.config.source_file);
                    resolved_path = parent_dir_path
                        .join(p)
                        .as_os_str()
                        .to_str()
                        .unwrap()
                        .to_string();
                }
                m_dbg!("resolved path: {}", resolved_path);
                lit(&resolved_path)
            }
            _ => path,
        };
        image(caption, path, size_spec)
    }
}
impl Process for ImageConverter<'_> {
    fn process(&mut self, exp: crate::syntax::Tree) -> crate::syntax::Tree {
        m_trc!("{:?}", self);
        self.walk(exp)
    }
}

pub fn new<'a>(c: &'a Config) -> Box<dyn Process + 'a> {
    Box::new(ImageConverter { config: c })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::syntax::{image, image_size, lit, DocType};
    use crate::Tree;
    use crate::Process;

    fn make_config(source_file: &str) -> Config {
        let mut c = Config::default();
        c.source_file = source_file.to_string();
        c
    }

    fn run(config: &Config, exp: Tree) -> Tree {
        let mut conv = ImageConverter { config };
        conv.process(exp)
    }

    // --- Path resolution ---

    #[test]
    fn absolute_path_is_unchanged() {
        let config = make_config("/docs/file.md");
        let input = Tree::Document(
            DocType::DEFAULT,
            Box::new(image(lit("alt"), lit("/images/pic.png"), image_size(lit("100"), lit("100")))),
        );
        let result = run(&config, input);
        // The absolute path should pass through unchanged
        assert!(format!("{result:?}").contains("\"/images/pic.png\""));
    }

    #[test]
    fn relative_path_is_joined_with_source_dir() {
        let config = make_config("/docs/subdir/file.md");
        let input = Tree::Document(
            DocType::DEFAULT,
            Box::new(image(lit("alt"), lit("pic.png"), image_size(lit("100"), lit("100")))),
        );
        let result = run(&config, input);
        // Relative path should be prefixed with the source file's parent directory
        assert!(format!("{result:?}").contains("/docs/subdir/pic.png"));
    }

    #[test]
    fn non_image_nodes_are_passed_through() {
        let config = make_config("/docs/file.md");
        let input = Tree::Document(DocType::DEFAULT, Box::new(lit("hello")));
        let result = run(&config, input);
        assert_eq!(format!("{result:?}"), "Document(DEFAULT, Literal(\"hello\"))");
    }

    #[test]
    fn cat_is_walked_recursively() {
        let config = make_config("/docs/file.md");
        let inner = image(lit("alt"), lit("pic.png"), image_size(lit("100"), lit("100")));
        let input = Tree::Document(DocType::DEFAULT, Box::new(lit("text").cat(inner)));
        let result = run(&config, input);
        // The relative path inside Cat should still be resolved
        assert!(format!("{result:?}").contains("/docs/pic.png"));
    }
}
