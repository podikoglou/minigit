use std::path::PathBuf;

use crate::storage::Store;

/// A Git repository, containing a `.git` directory.
pub struct Repo {
    path: PathBuf,
    pub store: Store,
}

impl Repo {
    /// Opens an already existing Git repository, constructing a [Repo].
    pub fn open(path: impl Into<PathBuf>) -> Result<Repo, anyhow::Error> {
        let path = path.into();

        let store = Store::open(path.join(".git"))?;

        Ok(Repo { path, store })
    }
}
