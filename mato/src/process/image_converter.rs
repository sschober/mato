use std::{collections::HashMap, path::Path};

use crate::{
    config::Config,
    syntax::{image, lit, Exp},
};

use super::Process;

/// ImageConverter processor currently only transforms
/// path information in the image expression.
/// Ultimately, we want it to do conversion and caching.
#[derive(Default)]
pub struct ImageConverter {}

impl ImageConverter {
    fn process_images(&mut self, exp: Exp, config: &Config) -> Exp {
        match exp {
            Exp::Cat(b1, b2) => self
                .process_images(*b1, config)
                .cat(self.process_images(*b2, config)),
            Exp::Image(caption, path) => {
                let path = match *path {
                    Exp::Literal(p) => {
                        let mut resolved_path = p.clone();
                        if !p.starts_with("/") {
                            let parent_dir_path = Path::new(&config.parent_dir);
                            resolved_path = parent_dir_path
                                .join(p)
                                .as_os_str()
                                .to_str()
                                .unwrap()
                                .to_string();
                        }
                        eprintln!("resolved path: {}", resolved_path);
                        lit(&resolved_path)
                    }
                    _ => *path,
                };
                image(*caption, path)
            }
            _ => exp,
        }
    }
}
impl Process for ImageConverter {
    fn process(
        &mut self,
        exp: crate::syntax::Exp,
        config: &crate::config::Config,
    ) -> crate::syntax::Exp {
        self.process_images(exp, config)
    }

    fn get_context(&mut self) -> std::collections::HashMap<String, String> {
        HashMap::new()
    }
}

pub fn new() -> Box<dyn Process> {
    Box::new(ImageConverter {})
}
