use super::{repo::Repo, template::Template, Result};

use directories::ProjectDirs;
use liquid;
use serde_yaml;

use std::{
    collections::HashSet,
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
        debug!("Saving repos ...");
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
                trace!(
                    "Removing repo at {:?} from repo list since it no longer exists",
                    repo.path
                );
                false
            } else {
                true
            }
        });
    }

    fn create_template_globals<'a>(&self, project_name: &'a str) -> liquid::Object {
        let mut globals = liquid::Object::new();

        macro_rules! fill_globals {
            ($($key:expr => $value:expr),*,) => {{
                $(
                    globals.insert(String::from($key), liquid::Value::scalar($value));
                )*
            }}
        }

        fill_globals!{
           "project_name" => project_name,
        }

        globals
    }

    pub fn create_repo<P: AsRef<Path>>(
        &mut self,
        project_name: P,
        template_name: &str,
    ) -> Result<()> {
        let project_name = project_name.as_ref();

        if project_name.exists() {
            bail!("Destination {:?} already exists.", project_name);
        }

        let parser = liquid::ParserBuilder::with_liquid().build();
        let template_path = self.template_dir.join(template_name.to_owned() + ".yaml");
        let template: Template = serde_yaml::from_reader(&fs::File::open(template_path)?)?;

        let repo_path = template.create(
            &parser,
            &self.create_template_globals(project_name.to_str().unwrap()),
        )?;

        self.repos.push(Repo {
            path: repo_path.canonicalize()?.to_owned(),
            labels: HashSet::new(),
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
            repo.labels.insert(label);
        }

        Ok(())
    }

    pub fn remove_label(&mut self, label: &str) -> Result<()> {
        let repo = self.nearest_repo()?;

        info!("Removing the label {:?} from the current repo.", label);
        if repo.labels.contains(label) {
            info!("Removing the label {:?} from the current repo.", label);
            repo.labels.remove(label);
        } else {
            bail!("The current repo doesn't have the label {:?}", label);
        }

        Ok(())
    }

    pub fn list_labels(&mut self) -> Result<&HashSet<String>> {
        let repo = self.nearest_repo()?;

        Ok(&repo.labels)
    }
}
