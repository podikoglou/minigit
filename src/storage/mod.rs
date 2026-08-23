use std::{
    fs::{self},
    io,
    path::PathBuf,
};

use anyhow::{Context, bail};

use crate::{
    hash::HashPrefix,
    storage::{object::LazyObject, prefix_dir::PrefixDir},
};

pub mod object;
pub mod prefix_dir;

pub struct Store {
    path: PathBuf,
}

impl Store {
    pub fn try_new(path: PathBuf) -> Result<Self, anyhow::Error> {
        match fs::exists(&path) {
            Ok(true) => Ok(Self { path }),
            Ok(false) => bail!("directory does not exist"),
            Err(err) => bail!(err),
        }
    }

    pub fn objects_path(&self) -> PathBuf {
        self.path.join("objects/")
    }

    /// Returns an iterator over the paths of the prefix directories.
    pub fn prefix_dirs(&self) -> Result<impl Iterator<Item = PrefixDir>, io::Error> {
        Ok(fs::read_dir(self.objects_path())?
            .into_iter()
            .filter_map(Result::ok)
            .filter_map(|entry| PrefixDir::try_new(entry.path()).ok()))
    }

    /// Finds a [PrefixDir] in .git/objects/
    pub fn prefix_dir(&self, prefix: HashPrefix) -> Result<PrefixDir, anyhow::Error> {
        self.prefix_dirs()?
            .into_iter()
            .find(|x| x.prefix == prefix)
            .context("couldn't find prefix dir")
    }

    pub fn objects(&self) -> Result<impl Iterator<Item = LazyObject>, anyhow::Error> {
        Ok(self
            .prefix_dirs()?
            .filter_map(|dir| dir.objects().ok())
            .flatten())
    }

    // /// Writes an object to the database.
    // pub fn add_object(&self, object: Object) -> Result<(), io::Error> {
    //     let hash = object.hash()?;
    //     let prefix = hash.prefix();
    // }
}
