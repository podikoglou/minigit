use std::{
    fs::{self},
    path::PathBuf,
    rc::Rc,
};

use itertools::Itertools;

use crate::{
    MinigitError,
    object::hash::HashPrefix,
    storage::object::{LazyObject, ObjectsBucket, packed::Packfile},
};

pub mod object;

#[derive(Debug)]
pub struct Store {
    pub path: PathBuf,
}

impl Store {
    /// Opens an already existing git database.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, MinigitError> {
        let path = path.into();

        match fs::exists(&path) {
            Ok(true) => Ok(Self { path }),
            Ok(false) => Err(MinigitError::NoGitDirectory),
            Err(err) => Err(err.into()),
        }
    }

    #[must_use]
    pub fn objects_path(&self) -> PathBuf {
        self.path.join("objects/")
    }

    #[must_use]
    pub fn packs_path(&self) -> PathBuf {
        self.objects_path().join("pack/")
    }

    /// Returns all buckets (directories named after the prefix of a hash) under `.git/objects/`.
    ///
    /// At most 256 of them exist, so they are collected eagerly.
    pub fn buckets(&self) -> Result<Vec<ObjectsBucket>, MinigitError> {
        fs::read_dir(self.objects_path())?
            .filter_map(|result| match result {
                Err(err) => Some(Err(err.into())),
                Ok(entry) if entry.file_name().len() == 2 => {
                    Some(ObjectsBucket::open(entry.path()))
                }
                Ok(_) => None,
            })
            .collect()
    }

    /// Finds an [`ObjectsBucket`] in .git/objects/
    pub fn bucket(&self, prefix: HashPrefix) -> Result<ObjectsBucket, MinigitError> {
        self.buckets()?
            .into_iter()
            .find(|dir| dir.prefix == prefix)
            .ok_or(MinigitError::BucketNotFound)
    }

    /// Gets all the [Packfile]s in the store.
    pub fn packs(&self) -> Result<Vec<Rc<Packfile>>, MinigitError> {
        let pack_dir = self.packs_path();

        if !pack_dir.exists() {
            return Ok(Vec::new());
        }

        let mut packs = Vec::new();

        for entry in fs::read_dir(pack_dir)? {
            let path = entry?.path();

            if path.extension().and_then(|ext| ext.to_str()) == Some("pack") {
                let idx_path = path.with_extension("idx");

                let idx = if idx_path.exists() {
                    Some(idx_path)
                } else {
                    None
                };

                packs.push(Rc::new(Packfile::open(path, idx)?));
            }
        }

        Ok(packs)
    }

    pub fn objects(
        &self,
    ) -> Result<impl Iterator<Item = Result<LazyObject, MinigitError>>, MinigitError> {
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
