use std::{fs, io::BufReader, path::PathBuf};

use crate::{
    MinigitError,
    error::ParserContext,
    object::{Object, hash::ObjectHash},
    storage::object::loose,
};

/// An object which has not been loaded yet.
///
/// At any given time it can be turned into a real [Object] using [`Self::into_object`].
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LazyObject {
    Loose(PathBuf),
}

impl LazyObject {
    #[must_use]
    pub fn loose(path: PathBuf) -> Self {
        Self::Loose(path)
    }

    /// Reads the full object.
    pub fn into_object(&self) -> Result<Object, MinigitError> {
        match self {
            LazyObject::Loose(path) => {
                // TODO: remove clones here?
                let file = fs::File::open(path)?;
                let reader = BufReader::new(file);

                loose::read_object_compressed(reader, ParserContext::File(path.clone()))
            }
        }
    }

    pub fn hash(&self) -> Result<ObjectHash, MinigitError> {
        match self {
            LazyObject::Loose(path) => path.try_into(),
        }
    }
}
