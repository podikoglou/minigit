use std::{
    fs::{self},
    path::Path,
};

use winnow::{ModalResult, Parser as _};

use crate::{
    ParserContext, TinygitError,
    parsing::{ParseResultExt, Stream},
};

/// Behaviour for contextfully decoding a value from some bytes.
///
/// The trait isn't unique to a specific format, but it must be the case that for this trait to be
/// implemented for a type, there is only *one* way to decode a value of that type.
pub trait Decode: Sized {
    /// The main parser of the type.
    ///
    /// Should generally not be used directly as it is contextless.
    fn parse(input: &mut Stream<'_>) -> ModalResult<Self>;

    /// Parses from a [Stream] with appropriate context.
    fn decode_stream(input: Stream, context: ParserContext) -> Result<Self, TinygitError> {
        Self::parse.parse(input).with_context(context)
    }

    /// Parses from a [Path] with appropriate context.
    fn decode_file(path: impl AsRef<Path>) -> Result<Self, TinygitError> {
        let path = path.as_ref();
        let bytes = fs::read(path)?;
        let stream = Stream::new(&bytes);

        Self::parse
            .parse(stream)
            .with_context(ParserContext::File(path.to_path_buf()))
    }

    /// Parses from a string slice, with no context.
    fn decode_str(input: &str) -> Result<Self, TinygitError> {
        let stream = Stream::new(input.as_bytes());

        Self::parse.parse(stream).with_context(ParserContext::None)
    }

    /// Parses from a string slice with appropriate context.
    fn decode_str_with_context(input: &str, context: ParserContext) -> Result<Self, TinygitError> {
        let stream = Stream::new(input.as_bytes());

        Self::parse.parse(stream).with_context(context)
    }
}
