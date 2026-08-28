use std::{fs, io, path::PathBuf};

use anyhow::{Context, bail};

use crate::object::{
    Object, ObjectType,
    hash::{HashPrefix, ObjectHash},
};

pub mod prefix_dir;

/// A lazily loaded object which has not been loaded yet.
///
/// The existence of a [LazyObject] struct ensures that the object actually exists in the object
/// database, or at least, that it existed during its creation.
///
/// At any given time it can be turned into a real [Object] using [`Self::into_object`].
pub struct LazyObject {
    pub path: PathBuf,
    pub hash: ObjectHash,
    pub prefix: HashPrefix,
}

impl LazyObject {
    /// Creates a new [LazyObject], validating that it exists.
    pub fn try_new(path: PathBuf) -> Result<Self, anyhow::Error> {
        match fs::exists(&path) {
            Ok(true) => {
                let prefix = path
                    .components()
                    .nth_back(1)
                    .context("couldn't find prefix")?
                    .as_os_str()
                    .to_string_lossy();

                let file_name = path.file_name().context("couldn't get file name")?;

                let hash: ObjectHash = format!("{prefix:?}{file_name:?}")
                    .try_into()
                    .context("couldn't parse object hash")?;

                let prefix = hash.prefix();

                Ok(Self { hash, prefix, path })
            }
            Ok(false) => bail!("file does not exist"),
            Err(err) => Err(err.into()),
        }
    }

    /// Reads the first few bytes to figure out the type of object.
    pub fn object_type(&self) -> Result<ObjectType, io::Error> {
        todo!()
    }

    /// Reads the full object.
    pub fn into_object(&self) -> Result<Object, io::Error> {
        todo!()
    }
}
