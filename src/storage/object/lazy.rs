use std::{fs, path::PathBuf};

use crate::{
    MinigitError,
    object::{
        Object,
        hash::{HashPrefix, ObjectHash},
    },
    storage::object::loose,
};

/// An object which has not been loaded yet.
///
/// The existence of a [LazyObject] struct ensures that the object actually exists in the object
/// database, or at least, that it existed during its creation.
///
/// At any given time it can be turned into a real [Object] using [`Self::into_object`].
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LazyObject {
    pub path: PathBuf,
    pub hash: ObjectHash,
    pub prefix: HashPrefix,
}

impl LazyObject {
    /// Creates a new [LazyObject], validating that it exists.
    pub fn try_new(path: PathBuf) -> Result<Self, MinigitError> {
        match fs::exists(&path) {
            Ok(true) => {
                let hash = ObjectHash::try_from(&path)?;
                let prefix = hash.prefix();

                Ok(Self { hash, prefix, path })
            }
            Ok(false) => Err(MinigitError::ObjectNotFound),
            Err(err) => Err(err.into()),
        }
    }

    /// Reads the full object.
    pub fn into_object(&self) -> Result<Object, MinigitError> {
        loose::read_object(&self.path)
    }
}
