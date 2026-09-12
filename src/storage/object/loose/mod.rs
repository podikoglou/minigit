//! This module deals with loose objects, i.e. objects in `.git/objects/`.

pub mod bucket;
pub mod parser;

pub use parser::parse_object;

use std::{fs, path::Path};

use crate::{MinigitError, object::Object};

/// Reads and parses an [Object] from a [PathBuf].
///
/// Objects are typically small enough, so this is not a streaming operation.
pub fn read_object(path: impl AsRef<Path>) -> Result<Object, MinigitError> {
    let contents = fs::read(path)?;

    parse_object(&contents)
}
