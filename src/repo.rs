use std::{fs, path::PathBuf};

use crate::{TinygitError, storage::Store};

/// A Git repository, containing a `.git` directory.
#[derive(Debug)]
pub struct Repo {
    pub path: PathBuf,
    pub store: Store,
}

impl Repo {
    /// Opens an already existing Git repository, constructing a [Repo].
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, TinygitError> {
        let path = path.into();

        let store = Store::open(path.join(".git"))?;

        Ok(Self { path, store })
    }

    /// Creates a new empty Git repository.
    pub fn new(path: impl Into<PathBuf>) -> Result<Repo, TinygitError> {
        let path = path.into();

        let git_dir = path.join(".git");

        // remove directory if it already exists (to re-initialize it)
        if git_dir.exists() {
            fs::remove_dir_all(git_dir)?;
        }

        fs::create_dir_all(path.join(".git/objects/pack"))?;
        fs::create_dir_all(path.join(".git/refs/heads"))?;

        let store = Store::open(path.join(".git"))?;
        // TODO: make HEAD reference

        Ok(Self { store, path })
    }
}
