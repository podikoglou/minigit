//! This module deals with loose objects, i.e. objects in `.git/objects/`.

pub mod bucket;
pub mod parser;

use crate::{MinigitError, object::Object};
use flate2::read::ZlibDecoder;
pub use parser::parse_object;
use std::{fs::File, io::Read, path::Path};

/// Reads, decompresses and parses an [Object] from a [PathBuf].
///
/// Objects are typically small enough, so this is not a streaming operation.
pub fn read_object(path: impl AsRef<Path>) -> Result<Object, MinigitError> {
    let file = File::open(path)?;
    let mut decoder = ZlibDecoder::new(file);

    let mut buf = Vec::new();
    decoder.read_to_end(&mut buf)?;

    parse_object(&buf)
}
