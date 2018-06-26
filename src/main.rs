extern crate clap;
use clap::{App, Arg, SubCommand};

extern crate handlebars;
use handlebars::Handlebars;

extern crate serde;

extern crate serde_yaml;

#[macro_use]
extern crate serde_derive;

extern crate failure;
type Result<T> = ::std::result::Result<T, failure::Error>;

#[macro_use]
extern crate log;

extern crate pretty_logger;

use std::{env, fs, path::Path, process};

#[derive(Deserialize, Debug)]
struct TemplateInfo {
    name: String,
    templates: Vec<Template>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum Template {
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
    fn create(&self, registry: &Handlebars, vars: &impl serde::Serialize) -> Result<()> {
        match self {
            Template::File { name, contents } => {
                let name = registry.render_template(name, vars)?;
                let contents = registry.render_template(contents, vars)?;
                info!("Creating file {:?}", name);
                std::fs::write(name, contents)?;
                Ok(())
            }

            Template::Directory { name, contents } => {
                let name = registry.render_template(name, vars)?;
                let old_wd = env::current_dir()?;
                info!("Creating directory {:?}", name);
                fs::create_dir_all(&name)?;
                env::set_current_dir(&name)?;
                contents
                    .iter()
                    .map(|template| template.create(registry, vars))
                    .collect::<Result<()>>()?;
                env::set_current_dir(old_wd)?;
                Ok(())
            }
        }
    }
}

impl TemplateInfo {
    fn create(&self, registry: &Handlebars, vars: &impl serde::Serialize) -> Result<()> {
        self.templates
            .iter()
            .map(|template| template.create(registry, vars))
            .collect()
    }
}

fn main() -> Result<()> {
    pretty_logger::init_level(log::LogLevelFilter::Info)?;

    // FIXME: This doesn't work if you run `repoman` from any directory except its own top-level
    // directory.
    let template_dir = Path::new("./templates");

    let app_matches = App::new("repoman")
        .version("0.1.0")
        .author("PurpleMyst")
        .subcommand(
            SubCommand::with_name("new")
                .about("Create a new project")
                .arg(Arg::with_name("project_name").required(true))
                .arg(
                    Arg::with_name("from")
                        .takes_value(true)
                        .value_name("template")
                        .long("from")
                        .help("Which template to use."),
                ),
        )
        .subcommand(
            SubCommand::with_name("label")
                .about("Label the current project")
                .subcommand(
                    SubCommand::with_name("add").arg(Arg::with_name("label").required(true)),
                )
                .subcommand(
                    SubCommand::with_name("remove").arg(Arg::with_name("label").required(true)),
                ),
        )
        .subcommand(
            SubCommand::with_name("batch")
                .about("Run a batch operation on many projects")
                .arg(Arg::with_name("label").required(true))
                .arg(Arg::with_name("command").required(true))
                .arg(Arg::with_name("arg").multiple(true)),
        )
        .get_matches();

    match app_matches.subcommand() {
        ("new", Some(sub_matches)) => {
            let project_name = Path::new(sub_matches.value_of("project_name").unwrap());
            let template = sub_matches.value_of("from").unwrap_or("default");

            if project_name.exists() {
                error!("Destination {:?} already exists.", project_name);

                process::exit(1);
            }

            fs::create_dir(project_name)?;

            let registry = handlebars::Handlebars::new();
            let mut template_vars = ::std::collections::HashMap::new();
            template_vars.insert("project_name", project_name.to_str().unwrap());
            template_vars.insert("author", "PurpleMyst");
            let template_info: TemplateInfo = serde_yaml::from_reader(&fs::File::open(
                template_dir.join(template.to_owned() + ".yaml"),
            )?)?;

            let old_wd = env::current_dir()?;
            env::set_current_dir(project_name)?;
            template_info.create(&registry, &template_vars)?;
            env::set_current_dir(old_wd)?;
        }

        ("label", Some(_sub_matches)) => unimplemented!("label"),

        ("batch", Some(_sub_matches)) => unimplemented!("batch"),

        ("", None) => {
            // TODO: Do something better here.
            eprintln!("{}", app_matches.usage());
            std::process::exit(1);
        }

        _ => unreachable!(),
    }

    Ok(())
}
