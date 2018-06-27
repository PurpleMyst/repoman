use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct Repo {
    pub path: PathBuf,
    pub labels: Vec<String>,
}

impl Repo {
    pub fn find_nearest<'a>(repos: &'a mut Vec<Repo>, directory: &Path) -> Option<&'a mut Repo> {
        repos
            .iter_mut()
            .find(|repo| directory.ancestors().any(|ancestor| repo.path == ancestor))
    }
}
