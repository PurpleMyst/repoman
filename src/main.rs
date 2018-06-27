extern crate clap;
use clap::{App, Arg, SubCommand};

extern crate handlebars;
use handlebars::Handlebars;

extern crate serde;

extern crate serde_yaml;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate failure;
type Result<T> = ::std::result::Result<T, failure::Error>;

#[macro_use]
extern crate log;

extern crate pretty_logger;

extern crate directories;

use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    process,
};

type Label = String;

#[derive(Serialize, Deserialize, Debug)]
struct Repo {
    path: PathBuf,
    labels: Vec<Label>,
}

fn find_by_path<'a>(repos: &'a mut Vec<Repo>, needle: &Path) -> Option<&'a mut Repo> {
    repos.into_iter().find(|repo| needle == repo.path)
}

#[derive(Serialize, Deserialize, Debug)]
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
    fn create(&self, registry: &Handlebars, vars: &impl serde::Serialize) -> Result<PathBuf> {
        match self {
            Template::File { name, contents } => {
                let name = PathBuf::from(registry.render_template(name, vars)?);
                let contents = registry.render_template(contents, vars)?;
                info!("Creating file {:?}", name);
                std::fs::write(&name, contents)?;
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

fn main() -> Result<()> {
    pretty_logger::init_level(log::LogLevelFilter::Trace)?;
    let project_dirs = directories::ProjectDirs::from("it", "PurpleMyst", "repoman");
    let config_dir = project_dirs.config_dir();
    let repos_file = config_dir.join("repos.yaml");
    let template_dir = config_dir.join("templates");

    if !template_dir.exists() {
        info!("Creating {:?}", template_dir);
        fs::create_dir_all(&template_dir)?;
        let prepackaged_template_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("templates");
        for entry in fs::read_dir(prepackaged_template_dir)? {
            let template_path = entry?.path();
            fs::copy(
                &template_path,
                template_dir.join(template_path.file_name().unwrap()),
            )?;
        }
    }

    let mut repos: Vec<Repo> = if repos_file.exists() {
        serde_yaml::from_reader(fs::File::open(&repos_file)?)?
    } else {
        Vec::new()
    };

    repos.retain(|repo| {
        if !repo.path.exists() {
            info!(
                "Removing repo at {:?} from repo list since it no longer exists",
                repo.path
            );
            false
        } else {
            true
        }
    });

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
                // TODO: `add/remove` should be multiple.
                .subcommand(
                    SubCommand::with_name("add").arg(Arg::with_name("label").required(true)),
                )
                .subcommand(
                    SubCommand::with_name("remove").arg(Arg::with_name("label").required(true)),
                )
                .subcommand(
                    SubCommand::with_name("list")
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

            let registry = handlebars::Handlebars::new();

            let mut template_vars = HashMap::new();
            template_vars.insert("project_name", project_name.to_str().unwrap());
            template_vars.insert("author", "PurpleMyst");

            let template: Template = serde_yaml::from_reader(&fs::File::open(
                template_dir.join(template.to_owned() + ".yaml"),
            )?)?;

            let repo_path = template.create(&registry, &template_vars)?;

            repos.push(Repo {
                path: repo_path.canonicalize()?.to_owned(),
                labels: Vec::new(),
            });
        }

        ("label", Some(sub_matches)) => {
            // TODO: Lookup parents too.
            let mut repo = find_by_path(&mut repos, &env::current_dir()?)
                .ok_or_else(|| format_err!("The current directory is not a repo."))?;

            match sub_matches.subcommand() {
                ("add", Some(sub_matches)) => {
                    let label = sub_matches.value_of("label").unwrap().to_owned();

                    if repo.labels.contains(&label) {
                        error!("The current repo already has the label {:?}", label);
                    } else {
                        info!("Adding the label {:?} to the current repo.", label);
                        repo.labels.push(label);
                    }
                }

                ("remove", Some(sub_matches)) => {
                    let label = sub_matches.value_of("label").unwrap();
                    info!("Removing the label {:?} from the current repo.", label);
                    repo.labels.retain(|l| l != label);
                }

                ("list", Some(_sub_matches)) => {
                    info!("The current repo has the labels {:?}", repo.labels);
                }

                ("", None) => {
                    // TODO: Do something better here.
                    eprintln!("{}", app_matches.usage());
                    std::process::exit(1);
                }
                _ => unreachable!(),
            }
        }

        ("batch", Some(_sub_matches)) => unimplemented!("batch"),

        ("", None) => {
            // TODO: Do something better here.
            eprintln!("{}", app_matches.usage());
            std::process::exit(1);
        }

        _ => unreachable!(),
    }

    trace!("Saving repos ...");
    serde_yaml::to_writer(fs::File::create(repos_file)?, &repos)?;

    Ok(())
}
