use super::Result;

use handlebars::Handlebars;
use serde::Serialize;

use std::{env, fs, path::PathBuf};

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Template {
    File {
        name: String,
        contents: String,
    },

    Directory {
        name: String,
        contents: Vec<Template>,
    },
}

impl Template {
    pub fn create(&self, registry: &Handlebars, vars: &impl Serialize) -> Result<PathBuf> {
        match self {
            Template::File { name, contents } => {
                let name = PathBuf::from(registry.render_template(name, vars)?);
                let contents = registry.render_template(contents, vars)?;
                info!("Creating file {:?}", name);
                fs::write(&name, contents)?;
                Ok(name)
            }

            Template::Directory { name, contents } => {
                let name = PathBuf::from(registry.render_template(name, vars)?);
                let old_wd = env::current_dir()?;
                info!("Creating directory {:?}", name);
                fs::create_dir_all(&name)?;
                env::set_current_dir(&name)?;
                contents
                    .iter()
                    .map(|template| template.create(registry, vars).map(|_| ()))
                    .collect::<Result<()>>()?;
                env::set_current_dir(old_wd)?;
                Ok(name)
            }
        }
    }
}
