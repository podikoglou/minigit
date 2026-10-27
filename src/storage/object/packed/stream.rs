use fallible_iterator::FallibleIterator;
use winnow::Parser;

use crate::{
    MinigitError,
    error::ParserContext,
    parsing::Stream,
    storage::object::packed::{PackedObject, object},
};

/// A [FallibleIterator] over the items of a packfile ([PackedObject]);
pub struct PackStream<'a> {
    /// The amount of objects to read. [`Self::next`] will start returning `None` after having read
    /// `amount` objects.
    amount: usize,

    /// The parser context, used for error reporting.
    context: ParserContext,

    /// A slice of the packfile's contents, starting from the first object.
    buf: Stream<'a>,

    /// (state) The amount of objects that have been read so far.
    read: usize,
}

impl<'a> PackStream<'a> {
    /// Creates a [PackStream] from a [Stream].
    ///
    /// This assumes the stream points to the first object, rather than the header of the Packfile.
    pub fn new_from_stream(stream: Stream<'a>, amount: usize, context: ParserContext) -> Self {
        PackStream {
            amount,
            context,
            buf: stream,
            read: 0,
        }
    }

    /// Creates a [PackStream] from a slice.
    ///
    /// This assumes the slice points to the first object, rather than the header of the Packfile.
    pub fn new_from_slice(slice: &'a [u8], amount: usize, context: ParserContext) -> Self {
        PackStream {
            amount,
            context,
            buf: Stream::new(slice),
            read: 0,
        }
    }
}

impl FallibleIterator for PackStream<'_> {
    type Item = PackedObject;
    type Error = MinigitError;

    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        if self.read >= self.amount {
            return Ok(None);
        }

        let object = object
            .parse_next(&mut self.buf)
            .map_err(|err| MinigitError::ParserError(err.to_string(), self.context.to_owned()))?;

        self.read += 1;

        Ok(Some(object))
    }
}
