use crate::{
    config::Config,
    m_dbg, m_trc,
    syntax::{image, lit, Tree},
};

use crate::Process;

/// ImageConverter processor currently only transforms
/// path information in the image expression.
/// Ultimately, we want it to do conversion and caching.
#[derive(Default, Debug)]
pub struct ImageConverter {}

fn process_images(exp: Tree, config: &Config) -> Tree {
    match exp {
        Tree::Document(dt, be) => Tree::Document(dt, Box::new(process_images(*be, config))),
        Tree::Cat(b1, b2) => process_images(*b1, config).cat(process_images(*b2, config)),
        Tree::Image(caption, path, size_spec) => {
            let path = match *path {
                Tree::Literal(p) => {
                    let mut resolved_path = p.clone();
                    if !p.starts_with('/') {
                        let parent_dir_path = crate::parent_dir(&config.source_file);
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
impl Process for ImageConverter {
    fn process(
        &mut self,
        exp: crate::syntax::Tree,
        config: &crate::config::Config,
    ) -> crate::syntax::Tree {
        m_trc!("{:?}", self);
        process_images(exp, config)
    }
}

pub fn new() -> Box<dyn Process> {
    Box::new(ImageConverter {})
}
