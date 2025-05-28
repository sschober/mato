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
