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
    fn process_images(&mut self, exp: Tree) -> Tree {
        match exp {
            Tree::Document(dt, be) => Tree::Document(dt, Box::new(self.process_images(*be))),
            Tree::Cat(b1, b2) => self.process_images(*b1).cat(self.process_images(*b2)),
            Tree::Image(caption, path, size_spec) => {
                let path = match *path {
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
                    _ => *path,
                };
                image(*caption, path, *size_spec)
            }
            _ => exp,
        }
    }
}
impl Process for ImageConverter<'_> {
    fn process(&mut self, exp: crate::syntax::Tree) -> crate::syntax::Tree {
        m_trc!("{:?}", self);
        self.process_images(exp)
    }
}

pub fn new<'a>(c: &'a Config) -> Box<dyn Process + 'a> {
    Box::new(ImageConverter { config: c })
}
