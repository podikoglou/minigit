use std::{
    fs::{self},
    path::PathBuf,
};

use anyhow::{Context, bail};
use itertools::Itertools;

use crate::{
    object::hash::HashPrefix,
    storage::object::{LazyObject, ObjectsBucket},
};

pub mod object;

pub struct Store {
    path: PathBuf,
}

impl Store {
    /// Opens an already existing git database.
    pub fn open(path: PathBuf) -> Result<Self, anyhow::Error> {
        match fs::exists(&path) {
            Ok(true) => Ok(Self { path }),
            Ok(false) => bail!("git directory does not exist"),
            Err(err) => bail!(err),
        }
    }

    pub fn objects_path(&self) -> PathBuf {
        self.path.join("objects/")
    }

    /// Returns all buckets (directories named after the prefix of a hash) under `.git/objects/`.
    ///
    /// At most 256 of them exist, so they are collected eagerly.
    pub fn buckets(&self) -> Result<Vec<ObjectsBucket>, anyhow::Error> {
        fs::read_dir(self.objects_path())?
            .filter_map(|result| match result {
                Err(err) => Some(Err(anyhow::format_err!("{err}"))),
                Ok(entry) if entry.file_name().len() == 2 => {
                    Some(ObjectsBucket::open(entry.path()))
                }
                Ok(_) => None,
            })
            .collect()
    }

    /// Finds an [ObjectsBucket] in .git/objects/
    pub fn bucket(&self, prefix: HashPrefix) -> Result<ObjectsBucket, anyhow::Error> {
        self.buckets()?
            .into_iter()
            .find(|dir| dir.prefix == prefix)
            .context("couldn't find prefix dir")
    }

    pub fn objects(
        &self,
    ) -> Result<impl Iterator<Item = Result<LazyObject, anyhow::Error>>, anyhow::Error> {
        Ok(self
            .buckets()?
            .into_iter()
            .map(ObjectsBucket::objects)
            .flatten_ok()
            .map(Result::flatten))
    }

    // /// Writes an object to the database.
    // pub fn add_object(&self, object: Object) -> Result<(), io::Error> {
    //     let hash = object.hash()?;
    //     let prefix = hash.prefix();
    // }
}
