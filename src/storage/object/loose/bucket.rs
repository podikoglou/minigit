//! This module deals with buckets in a loose object store.
//!
//! In the loose object database (`.git/objects/`), objects are indexed by their
//! [`crate::object::hash::ObjectHash`] represented in hexadecimal. In particular, they are placed in
//! *buckets* named after the [`HashPrefix`], (i.e. the first two characters of the hexadecimal hash),
//! and the prefix is removed from the object file name.
//!
//! ```txt
//! .git/objects
//! ├── 00
//! │   ├── 77a275f2a44ea4c1ea187e9bbb95998a468e43
//! │   ├── 87288858ff994e811024ebe37e0035fafad790
//! │   ├── f856dd6c92aec1cbd77b2204cf409d47580cb5
//! │   └
//! ```
//!
//! In this object database for example, there exist three objects with the following hashes:
//! - `0077a275f2a44ea4c1ea187e9bbb95998a468e43`
//! - `0087288858ff994e811024ebe37e0035fafad790`
//! - `00f856dd6c92aec1cbd77b2204cf409d47580cb5`
use std::{fs, path::PathBuf};

use crate::{MinigitError, object::hash::HashPrefix, storage::object::lazy::LazyObject};

/// A directory containing objects which start with a certain prefix, placed under `.git/objects/`.
pub struct ObjectsBucket {
    pub path: PathBuf,
    pub prefix: HashPrefix,
}

impl ObjectsBucket {
    /// Tries to open a [`ObjectsBucket`], validating that it exists.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, MinigitError> {
        let path = path.into();

        let name = path
            .file_name()
            .map(|name| name.to_string_lossy())
            .ok_or(MinigitError::InvalidFileName)?;

        let prefix = name.parse()?;

        match fs::exists(&path) {
            Ok(true) => Ok(Self { path, prefix }),
            Ok(false) => Err(MinigitError::BucketNotFound),
            Err(err) => Err(err.into()),
        }
    }

    /// Returns a lazy iterator over the [`LazyObject`]s in this bucket.
    ///
    /// A bucket can hold thousands of objects, so its contents are not read eagerly.
    ///
    /// Takes `self` by value: the iterator streams from the directory, so the
    /// caller gives the handle away.
    pub fn objects(
        self,
    ) -> Result<impl Iterator<Item = Result<LazyObject, MinigitError>>, MinigitError> {
        Ok(fs::read_dir(self.path)?.filter_map(|path| match path {
            Err(err) => Some(Err(err.into())),
            Ok(entry) if entry.file_name().len() == 38 => Some(Ok(LazyObject::loose(entry.path()))),
            Ok(_) => None,
        }))
    }
}
