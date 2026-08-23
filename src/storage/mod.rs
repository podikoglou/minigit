use std::{
    fs::{self},
    io,
    path::PathBuf,
};

use anyhow::{Context, bail};
use itertools::Itertools;

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
    pub fn prefix_dirs(
        &self,
    ) -> Result<impl Iterator<Item = Result<PrefixDir, anyhow::Error>>, io::Error> {
        Ok(fs::read_dir(self.objects_path())?.map(|entry| PrefixDir::try_new(entry?.path())))
    }

    /// Finds a [PrefixDir] in .git/objects/
    pub fn prefix_dir(&self, prefix: HashPrefix) -> Result<PrefixDir, anyhow::Error> {
        self.prefix_dirs()?
            .into_iter()
            .find_map(|prefix_dir| {
                prefix_dir
                    .map(|dir| (dir.prefix == prefix).then_some(dir))
                    .transpose()
            })
            .context("couldn't find prefix dir")?
    }

    pub fn objects(
        &self,
    ) -> Result<impl Iterator<Item = Result<LazyObject, anyhow::Error>>, anyhow::Error> {
        Ok(self
            .prefix_dirs()?
            .map(|prefix_dir| PrefixDir::objects(prefix_dir?))
            .flatten_ok()
            .map(Result::flatten))
    }

    // /// Writes an object to the database.
    // pub fn add_object(&self, object: Object) -> Result<(), io::Error> {
    //     let hash = object.hash()?;
    //     let prefix = hash.prefix();
    // }
}
