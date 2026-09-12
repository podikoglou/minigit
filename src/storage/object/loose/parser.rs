//! This module deals with parsing loose object files. The most important function is [`parse_object`].
//!
//! This module contains several incremental parsers built using the `winnow` parser combinator
//! crate. It should be stressed that they will not fail if they have excess input, as they are
//! incremental and built to be combined.

use winnow::{
    ModalResult, Parser,
    ascii::dec_uint,
    combinator::{alt, seq},
    error::{ContextError, ErrMode, StrContext, StrContextValue},
    token::{literal, rest, take},
};

use crate::{
    MinigitError,
    object::{Object, ObjectType, blob::Blob},
};

/// Parses an object type string from some bytes.
pub fn object_type(input: &mut &[u8]) -> ModalResult<ObjectType> {
    alt((
        literal("blob").map(|_| ObjectType::Blob),
        literal("tree").map(|_| ObjectType::Tree),
    ))
    .context(StrContext::Expected(StrContextValue::Description("type")))
    .parse_next(input)
}

/// Given an input (which it consumes), read the object type and size of the rest of the object
pub fn header(input: &mut &[u8]) -> ModalResult<(ObjectType, usize)> {
    let mut size = dec_uint::<_, usize, ErrMode<ContextError>>.context(StrContext::Expected(
        StrContextValue::Description("payload size"),
    ));

    seq!(object_type, _: " ", size, _: "\0")
        .context(StrContext::Expected(StrContextValue::Description("header")))
        .parse_next(input)
}

/// Parses an object from some input.
pub fn object(input: &mut &[u8]) -> ModalResult<Object> {
    let (typee, size) = header.parse_next(input)?;
    let mut bytes = take(size).parse_next(input)?;

    match typee {
        ObjectType::Blob => blob.map(Object::Blob).parse_next(&mut bytes),
        ObjectType::Tree => todo!(),
    }
}

/// Parses a blob object's content.
pub fn blob(input: &mut &[u8]) -> ModalResult<Blob> {
    rest.map(|e: &[u8]| Blob(e.into())).parse_next(input)
}

/// High level function to parse an [Object] from some bytes.
pub fn parse_object(input: &[u8]) -> Result<Object, MinigitError> {
    object
        .parse(input)
        .map_err(|err| MinigitError::ParserError(err.to_string()))
}

#[cfg(test)]
mod tests {
    use crate::{
        object::{
            Object::{self},
            ObjectType,
            blob::Blob,
        },
        storage::object::loose::parser::{header, object, object_type},
    };
    use std::assert_matches;
    use winnow::{Parser, error::ErrMode};

    #[test]
    fn object_type_parses_expected_object_types() {
        assert_eq!(
            object_type.parse_peek(b"blob"),
            Ok((&b""[..], ObjectType::Blob))
        );
        assert_eq!(
            object_type.parse_peek(b"tree"),
            Ok((&b""[..], ObjectType::Tree))
        );
    }

    #[test]
    fn object_type_rejects_invalid_input() {
        assert_matches!(object_type.parse_peek(b""), Err(ErrMode::Backtrack(_)));
        assert_matches!(object_type.parse_peek(b"blo"), Err(ErrMode::Backtrack(_)));
    }

    #[test]
    fn header_parses_basic_headers() {
        assert_eq!(
            header.parse_peek(b"blob 3\0"),
            Ok((&b""[..], (ObjectType::Blob, 3)))
        );
        assert_eq!(
            header.parse_peek(b"tree 333\0"),
            Ok((&b""[..], (ObjectType::Tree, 333)))
        );
    }

    #[test]
    fn header_rejets_invalid_input() {
        assert_matches!(header.parse_peek(b"tre 3"), Err(ErrMode::Backtrack(_)));
        assert_matches!(header.parse_peek(b"tree "), Err(ErrMode::Backtrack(_)));
        assert_matches!(header.parse_peek(b"tree \0"), Err(ErrMode::Backtrack(_)));
        assert_matches!(header.parse_peek(b"tree\0"), Err(ErrMode::Backtrack(_)));
        assert_matches!(header.parse_peek(b"3"), Err(ErrMode::Backtrack(_)));
        assert_matches!(header.parse_peek(b"3\0"), Err(ErrMode::Backtrack(_)));
    }

    #[test]
    fn object_parses_basic_objects() {
        assert_eq!(
            object.parse_peek(b"blob 3\0\x03\x03\x01"),
            Ok((&b""[..], Object::Blob(Blob(vec![0x03, 0x03, 0x01]))))
        );
    }
}
