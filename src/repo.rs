use std::path::PathBuf;

use crate::storage::Store;

/// A Git repository, containing a `.git` directory.
pub struct Repo {
    path: PathBuf,
    pub store: Store,
}

impl Repo {
    /// Opens an already existing Git repository, constructing a [Repo].
    pub fn open(path: PathBuf) -> Result<Repo, anyhow::Error> {
        let store = Store::open(path.join(".git"))?;

        Ok(Repo { path, store })
    }
}
