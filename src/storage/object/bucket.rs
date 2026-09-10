use std::{fs, path::PathBuf};

use anyhow::{Context, bail};

use crate::{object::hash::HashPrefix, storage::object::lazy::LazyObject};

/// A directory containing objects which start with a certain prefix, placed under `.git/objects/`.
pub struct ObjectsBucket {
    path: PathBuf,
    pub prefix: HashPrefix,
}

impl ObjectsBucket {
    /// Tries to create a new [ObjectsBucket], validating that it exists.
    pub fn try_new(path: PathBuf) -> Result<ObjectsBucket, anyhow::Error> {
        let name = path
            .file_name()
            .context("couldn't get file name")
            .map(|name| name.to_string_lossy())?;

        let prefix = HashPrefix::try_from(name)?;

        match fs::exists(&path) {
            Ok(true) => Ok(Self { path, prefix }),
            Ok(false) => bail!("couldn't find prefix dir"),
            Err(err) => bail!(err),
        }
    }

    /// Returns a lazy iterator over the [LazyObject]s in this bucket.
    ///
    /// A bucket can hold thousands of objects, so its contents are not read eagerly.
    ///
    /// Takes `self` by value: the iterator streams from the directory, so the
    /// caller gives the handle away.
    pub fn objects(
        self,
    ) -> Result<impl Iterator<Item = Result<LazyObject, anyhow::Error>>, anyhow::Error> {
        Ok(fs::read_dir(self.path)?.map(|path| LazyObject::try_new(path?.path())))
    }
}
