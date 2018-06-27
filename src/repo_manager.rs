use super::{repo::Repo, template::Template, Result};

use directories::ProjectDirs;
use handlebars::Handlebars;
use serde_yaml;

use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct RepoManager {
    pub repos: Vec<Repo>,
    pub repos_file: PathBuf,
    pub template_dir: PathBuf,
}

impl Drop for RepoManager {
    fn drop(&mut self) {
        trace!("Saving repos ...");
        serde_yaml::to_writer(fs::File::create(&self.repos_file).unwrap(), &self.repos).unwrap();
    }
}

impl RepoManager {
    pub fn new() -> Result<Self> {
        let project_dirs = ProjectDirs::from("it", "PurpleMyst", "repoman");
        let config_dir = project_dirs.config_dir();
        let repos_file = config_dir.join("repos.yaml");
        let template_dir = config_dir.join("templates");

        Self::create_template_dir_if_needed(&template_dir)?;

        let mut repos = serde_yaml::from_reader(fs::File::open(&repos_file)?)?;
        Self::prune_removed_repos(&mut repos);

        Ok(RepoManager {
            repos,
            repos_file,
            template_dir,
        })
    }

    fn create_template_dir_if_needed(template_dir: &Path) -> Result<()> {
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

        Ok(())
    }

    fn prune_removed_repos(repos: &mut Vec<Repo>) {
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
    }

    fn create_template_vars<'a>(&self, project_name: &'a str) -> HashMap<&'static str, &'a str> {
        let mut template_vars = HashMap::new();
        template_vars.insert("project_name", project_name);
        template_vars.insert("author", "PurpleMyst");
        template_vars
    }

    pub fn create_repo(&mut self, project_name: &Path, template_name: &str) -> Result<()> {
        if project_name.exists() {
            bail!("Destination {:?} already exists.", project_name);
        }

        let registry = Handlebars::new();
        let template_path = self.template_dir.join(template_name.to_owned() + ".yaml");
        let template: Template = serde_yaml::from_reader(&fs::File::open(template_path)?)?;

        let repo_path = template.create(
            &registry,
            &self.create_template_vars(project_name.to_str().unwrap()),
        )?;

        self.repos.push(Repo {
            path: repo_path.canonicalize()?.to_owned(),
            labels: Vec::new(),
        });

        Ok(())
    }

    fn nearest_repo(&mut self) -> Result<&mut Repo> {
        Repo::find_nearest(&mut self.repos, &env::current_dir()?).ok_or_else(|| {
            format_err!("The current directory or any of its parents are not a repo.")
        })
    }

    pub fn add_label(&mut self, label: String) -> Result<()> {
        let repo = self.nearest_repo()?;

        if repo.labels.contains(&label) {
            bail!("The current repo already has the label {:?}", label);
        } else {
            info!("Adding the label {:?} to the current repo.", label);
            repo.labels.push(label);
        }

        Ok(())
    }

    pub fn remove_label(&mut self, label: &str) -> Result<()> {
        let repo = self.nearest_repo()?;

        info!("Removing the label {:?} from the current repo.", label);
        repo.labels.retain(|l| l != label);

        Ok(())
    }

    pub fn list_labels(&mut self) -> Result<&Vec<String>> {
        let repo = self.nearest_repo()?;

        Ok(&repo.labels)
    }
}
