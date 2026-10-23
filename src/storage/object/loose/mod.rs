//! This module deals with loose objects, i.e. objects in `.git/objects/`.

pub mod bucket;

use crate::{
    MinigitError,
    error::ParserContext,
    object::{Object, parse_object},
};
use flate2::read::ZlibDecoder;
use std::io::{BufRead, Read, Write};

/// Behaviour for encoding a value in the relevant format.
///
/// The trait isn't unique to a specific format, but it must be the case that for this trait to be
/// implemented for a type, there is only *one* way to encode a value of that type.
pub trait Encode {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), MinigitError>;
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
