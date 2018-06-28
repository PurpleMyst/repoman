use super::Result;

use liquid;

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
    pub fn create(&self, parser: &liquid::Parser, globals: &liquid::Object) -> Result<PathBuf> {
        match self {
            Template::File { name, contents } => {
                trace!("Rendering name template {:?} ...", name);
                let name = parser.parse(name)?.render(&globals)?;
                trace!("Rendering contents template {:?} ...", contents);
                let contents = parser.parse(contents)?.render(&globals)?;
                info!("Creating file {:?}", name);
                fs::write(&name, contents)?;
                Ok(PathBuf::from(name))
            }

            Template::Directory { name, contents } => {
                trace!("Trying to parse and render name template {:?}", name);
                let name = parser.parse(name)?.render(&globals)?;
                let old_wd = env::current_dir()?;
                info!("Creating directory {:?}", name);
                fs::create_dir_all(&name)?;
                env::set_current_dir(&name)?;
                for template in contents {
                    template.create(parser, globals)?;
                }
                env::set_current_dir(old_wd)?;
                Ok(PathBuf::from(name))
            }
        }
    }
}
