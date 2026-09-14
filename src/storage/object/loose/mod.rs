//! This module deals with loose objects, i.e. objects in `.git/objects/`.

pub mod bucket;
pub mod parser;

use crate::{MinigitError, error::ParserContext, object::Object};
use flate2::read::ZlibDecoder;
pub use parser::parse_object;
use std::io::{BufRead, Read};

/// Reads, decompresses and parses an [Object] from a [`Read`].
///
/// Objects are typically small enough, so this is not a streaming operation.
pub fn read_object(reader: impl BufRead, context: ParserContext) -> Result<Object, MinigitError> {
    let mut decoder = ZlibDecoder::new(reader);

    let mut buf = Vec::new();
    decoder.read_to_end(&mut buf)?;

    parse_object(&buf, context)
}
