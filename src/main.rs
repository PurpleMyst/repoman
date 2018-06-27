// TODO: Find a stable replacement for this.
#![feature(path_ancestors)]

extern crate clap;
use clap::{App, Arg, SubCommand};

extern crate handlebars;

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

mod repo;

mod template;

mod repo_manager;
use repo_manager::RepoManager;

fn main() -> Result<()> {
    pretty_logger::init_level(log::LogLevelFilter::Trace)?;

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
                    SubCommand::with_name("add")
                        .arg(Arg::with_name("label").required(true).multiple(true)),
                )
                .subcommand(
                    SubCommand::with_name("remove")
                        .arg(Arg::with_name("label").required(true).multiple(true)),
                )
                .subcommand(SubCommand::with_name("list")),
        )
        .subcommand(
            SubCommand::with_name("batch")
                .about("Run a batch operation on many projects")
                .arg(Arg::with_name("label").required(true))
                .arg(Arg::with_name("command").required(true))
                .arg(Arg::with_name("arg").multiple(true)),
        )
        .get_matches();

    let mut repo_manager = RepoManager::new()?;

    match app_matches.subcommand() {
        ("new", Some(sub_matches)) => {
            let project_name = sub_matches.value_of("project_name").unwrap();
            let template_name = sub_matches.value_of("from").unwrap_or("default");
            repo_manager.create_repo(project_name, template_name)?;
        }

        ("label", Some(sub_matches)) => {
            match sub_matches.subcommand() {
                ("add", Some(sub_matches)) => {
                    let labels = sub_matches.values_of("label").unwrap();
                    for label in labels {
                        repo_manager.add_label(label.to_owned())?;
                    }
                }

                ("remove", Some(sub_matches)) => {
                    let labels = sub_matches.values_of("label").unwrap();
                    for label in labels {
                        repo_manager.remove_label(label)?;
                    }
                }

                ("list", Some(_sub_matches)) => {
                    info!(
                        "The current repo has the labels {:?}",
                        repo_manager.list_labels()?
                    );
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

    Ok(())
}
