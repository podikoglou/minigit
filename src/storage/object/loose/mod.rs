//! This module deals with loose objects, i.e. objects in `.git/objects/`.

pub mod bucket;
pub mod parser;

use crate::{
    MinigitError,
    error::ParserContext,
    object::{Object, parse_object},
};
use flate2::read::ZlibDecoder;
use std::io::{BufRead, Read, Write};

/// Behaviour for encoding the struct in Git's loose object format.
pub trait WriteLoose {
    fn write_loose<W: Write>(&self, writer: &mut W) -> Result<(), MinigitError>;
}

/// Reads and parses an [Object] from a [`Read`].
///
/// Objects are typically small enough, so this is not a streaming operation.
pub fn read_object(
    mut reader: impl BufRead,
    context: ParserContext,
) -> Result<Object, MinigitError> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;

    parse_object(&buf, context)
}

/// Reads, decompresses and parses an [Object] from a [`Read`].
///
/// Objects are typically small enough, so this is not a streaming operation.
pub fn read_object_compressed(
    reader: impl BufRead,
    context: ParserContext,
) -> Result<Object, MinigitError> {
    let mut decoder = ZlibDecoder::new(reader);

    let mut buf = Vec::new();
    decoder.read_to_end(&mut buf)?;

    parse_object(&buf, context)
}
