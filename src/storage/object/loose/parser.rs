//! This module deals with parsing loose object files. The most important function is [`parse_object`].
//!
//! This module contains several incremental parsers built using the `winnow` parser combinator
//! crate. It should be stressed that they will not fail if they have excess input, as they are
//! incremental and built to be combined.

use sha1::digest::{array::Array, consts::U20};
use winnow::{
    ModalResult, Parser,
    ascii::{dec_uint, newline, oct_digit1, space1, till_line_ending},
    combinator::{alt, seq, terminated},
    error::{ContextError, ErrMode, StrContext, StrContextValue},
    token::{literal, rest, take},
};

use crate::{
    MinigitError,
    object::{Object, ObjectType, blob::Blob, hash::ObjectHash, tree::TreeEntry},
};

/// Parses an object type string from some bytes.
pub fn object_type(input: &mut &[u8]) -> ModalResult<ObjectType> {
    alt((
        literal("blob").map(|_| ObjectType::Blob),
        literal("tree").map(|_| ObjectType::Tree),
    ))
    .context(StrContext::Label("type"))
    .context(StrContext::Expected(StrContextValue::Description(
        "blob | tree",
    )))
    .parse_next(input)
}

/// Given an input (which it consumes), read the object type and size of the rest of the object
pub fn header(input: &mut &[u8]) -> ModalResult<(ObjectType, usize)> {
    let mut size = dec_uint::<_, usize, ErrMode<ContextError>>
        .context(StrContext::Label("payload size"))
        .context(StrContext::Expected(StrContextValue::Description(
            "bytes amount",
        )));

    seq!(object_type, _: " ", size, _: "\0")
        .context(StrContext::Label("header"))
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

/// Parses a file mode such as 100644, used in the tree
pub fn mode(input: &mut &[u8]) -> ModalResult<u16> {
    oct_digit1
        .map(str::from_utf8)
        .verify_map(Result::ok)
        .map(|str| u16::from_str_radix(str, 8))
        .verify_map(Result::ok)
        .parse_next(input)
}

/// Parses a hash (binary-encoded, as per how trees are encoded)
pub fn object_hash(input: &mut &[u8]) -> ModalResult<ObjectHash> {
    take(20usize)
        .map(Array::try_from)
        .verify_map(Result::ok)
        .map(ObjectHash::from)
        .parse_next(input)
}

/// Parses a file name in a tree entry.
///
/// Due to the format tree entry format, this reads until a newline.
pub fn file_name<'a>(input: &mut &'a [u8]) -> ModalResult<&'a str> {
    terminated(
        till_line_ending.map(str::from_utf8).verify_map(Result::ok),
        newline,
    )
    .parse_next(input)
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
        storage::object::loose::parser::{file_name, header, mode, object, object_type},
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
    fn mode_parses_valid_modes() {
        assert_eq!(mode.parse_peek(b"000000"), Ok((&b""[..], 0)));
        assert_eq!(mode.parse_peek(b"100644"), Ok((&b""[..], 0o100644)));
    }

    #[test]
    fn file_name_parses_valid_inputs() {
        assert_eq!(
            file_name.parse_peek(b"foo.bar\n"),
            Ok((&b""[..], "foo.bar"))
        );

        assert_eq!(
            file_name.parse_peek(b"even this!!\n"),
            Ok((&b""[..], "even this!!"))
        );
    }

    #[test]
    fn file_name_rejects_invalid_inputs() {
        assert_matches!(file_name.parse_peek(b"foo.bar"), Err(ErrMode::Backtrack(_)));
    }

    #[test]
    fn object_parses_basic_objects() {
        assert_eq!(
            object.parse_peek(b"blob 3\0\x03\x03\x01"),
            Ok((&b""[..], Object::Blob(Blob(vec![0x03, 0x03, 0x01]))))
        );
    }
}
