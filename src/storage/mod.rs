use std::{
    fs::{self},
    path::PathBuf,
};

use anyhow::{Context, bail};
use itertools::Itertools;

use crate::{
    object::hash::HashPrefix,
    storage::object::{LazyObject, prefix_dir::PrefixDir},
};

pub mod object;

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

    /// Returns all prefix directories under `.git/objects/`.
    ///
    /// At most 256 of them exist, so they are collected eagerly.
    pub fn prefix_dirs(&self) -> Result<Vec<PrefixDir>, anyhow::Error> {
        fs::read_dir(self.objects_path())?
            .map(|entry| PrefixDir::try_new(entry?.path()))
            .collect()
    }

    /// Finds a [PrefixDir] in .git/objects/
    pub fn prefix_dir(&self, prefix: HashPrefix) -> Result<PrefixDir, anyhow::Error> {
        self.prefix_dirs()?
            .into_iter()
            .find(|dir| dir.prefix == prefix)
            .context("couldn't find prefix dir")
    }

    pub fn objects(
        &self,
    ) -> Result<impl Iterator<Item = Result<LazyObject, anyhow::Error>>, anyhow::Error> {
        Ok(self
            .prefix_dirs()?
            .into_iter()
            .map(PrefixDir::objects)
            .flatten_ok()
            .map(Result::flatten))
    }

    // /// Writes an object to the database.
    // pub fn add_object(&self, object: Object) -> Result<(), io::Error> {
    //     let hash = object.hash()?;
    //     let prefix = hash.prefix();
    // }
}
