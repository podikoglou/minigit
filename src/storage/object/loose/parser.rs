//! This module deals with parsing loose object files. The most important function is [`parse_object`].
//!
//! This module contains several incremental parsers built using the `winnow` parser combinator
//! crate. It should be stressed that they will not fail if they have excess input, as they are
//! incremental and built to be combined.

use std::collections::BTreeMap;

use chrono::{DateTime, FixedOffset};
use sha1::digest::array::Array;
use winnow::{
    ModalResult, Parser,
    ascii::{dec_uint, oct_digit1},
    combinator::{alt, repeat, seq, terminated},
    error::{ContextError, ErrMode, StrContext, StrContextValue},
    token::{literal, rest, take, take_until},
};

use crate::{
    MinigitError,
    object::{
        Object, ObjectType,
        blob::Blob,
        commit::{Commit, Identity},
        hash::ObjectHash,
        tree::{Tree, TreeEntry},
    },
};

/// Parses an object type string from some bytes.
pub fn object_type(input: &mut &[u8]) -> ModalResult<ObjectType> {
    alt((
        literal("blob").map(|_| ObjectType::Blob),
        literal("tree").map(|_| ObjectType::Tree),
        literal("commit").map(|_| ObjectType::Commit),
    ))
    .context(StrContext::Label("type"))
    .context(StrContext::Expected(StrContextValue::Description(
        "blob | tree | commit",
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
        ObjectType::Tree => tree.map(Object::Tree).parse_next(&mut bytes),
        ObjectType::Commit => commit.map(Object::Commit).parse_next(&mut bytes),
    }
}

/// Parses a blob object's content.
pub fn blob(input: &mut &[u8]) -> ModalResult<Blob> {
    rest.map(|e: &[u8]| Blob(e.into())).parse_next(input)
}

pub fn tree(input: &mut &[u8]) -> ModalResult<Tree> {
    // NOTE: not sure if this should be `0..` or `1..`
    // should we be able to parse empty trees?
    repeat(0.., tree_entry)
        .map(|entries: Vec<(u16, &str, ObjectHash)>| {
            entries
                .into_iter()
                .map(|(mode, name, hash)| (name.to_string(), TreeEntry::new(mode, hash)))
                .collect::<BTreeMap<String, TreeEntry>>()
        })
        .map(Tree::new)
        .parse_next(input)
}

/// Parses a tree object's entry into a tuple `(mode, name, hash)`
pub fn tree_entry<'a>(input: &mut &'a [u8]) -> ModalResult<(u16, &'a str, ObjectHash)> {
    seq!((mode, _: " ", file_name, object_hash)).parse_next(input)
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

/// Parses a hash (binary-encoded, as per how trees are encoded)
pub fn object_hash_str(input: &mut &[u8]) -> ModalResult<ObjectHash> {
    take(40usize)
        .map(str::from_utf8)
        .verify_map(Result::ok)
        .map(str::parse::<ObjectHash>)
        .verify_map(Result::ok)
        .parse_next(input)
}

/// Parses a file name in a tree entry.
///
/// Due to the format tree entry format, this reads until a NUL character.
pub fn file_name<'a>(input: &mut &'a [u8]) -> ModalResult<&'a str> {
    terminated(
        take_until(1.., 0x00)
            .map(str::from_utf8)
            .verify_map(Result::ok),
        0x00,
    )
    .parse_next(input)
}

/// High level function to parse an [Object] from some bytes.
pub fn parse_object(input: &[u8]) -> Result<Object, MinigitError> {
    object
        .parse(input)
        .map_err(|err| MinigitError::ParserError(err.to_string()))
}

pub fn identity(input: &mut &[u8]) -> ModalResult<Identity> {
    seq! {Identity{
        _: "author",
        name: take_until(1.., " <").map(str::from_utf8).verify_map(Result::ok).map(str::to_owned),
        _: " <",
        email: take_until(1.., ">").map(str::from_utf8).verify_map(Result::ok).map(str::to_owned),
        _: "> ",
    }}
    .parse_next(input)
}

pub fn timestamp(input: &mut &[u8]) -> ModalResult<DateTime<FixedOffset>> {
    take_until(0.., "\n")
        .map(str::from_utf8)
        .verify_map(Result::ok)
        .map(|f| DateTime::parse_from_str(f, "%s %z"))
        .verify_map(Result::ok)
        .parse_next(input)
}

pub fn commit(input: &mut &[u8]) -> ModalResult<Commit> {
    seq! {Commit{
        _: "tree ",
        tree: object_hash_str,

        author: seq!(identity, _: " ", timestamp),
        committer: seq!(identity, _: " ", timestamp),

        description: rest.map(str::from_utf8).verify_map(Result::ok).map(str::to_owned)
    }}
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use crate::{
        object::{
            Object::{self},
            ObjectType,
            blob::Blob,
        },
        storage::object::loose::parser::{
            file_name, header, mode, object, object_hash_str, object_type, tree_entry,
        },
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
            file_name.parse_peek(b"foo.bar\0"),
            Ok((&b""[..], "foo.bar"))
        );

        assert_eq!(
            file_name.parse_peek(b"even this!!\0"),
            Ok((&b""[..], "even this!!"))
        );
    }

    #[test]
    fn file_name_rejects_invalid_inputs() {
        assert_matches!(file_name.parse_peek(b"foo.bar"), Err(ErrMode::Backtrack(_)));
        assert_matches!(
            file_name.parse_peek(b"foo.bar\n"),
            Err(ErrMode::Backtrack(_))
        );
    }

    #[test]
    fn tree_entry_parses_valid_entries() {
        assert_eq!(
            tree_entry.parse_peek(b"100644 cli.rs\0\x29\xf3\x23\xb3\x1a\xd1\x29\x96\x4f\xfb\x4f\x97\xf2\x03\xbe\x9c\x2f\x35\x10\x7d"),
            Ok((&b""[..], (0o100644, "cli.rs", [0x29, 0xf3, 0x23, 0xb3, 0x1a, 0xd1, 0x29, 0x96, 0x4f, 0xfb, 0x4f, 0x97, 0xf2, 0x03, 0xbe, 0x9c, 0x2f, 0x35, 0x10, 0x7d].into() )))
        );
    }

    #[test]
    fn object_hash_str_parses_valid_hashes() {
        assert_eq!(
            object_hash_str.parse_peek(b"29f323b31ad129964ffb4f97f203be9c2f35107d"),
            Ok((
                &b""[..],
                [
                    0x29, 0xf3, 0x23, 0xb3, 0x1a, 0xd1, 0x29, 0x96, 0x4f, 0xfb, 0x4f, 0x97, 0xf2,
                    0x03, 0xbe, 0x9c, 0x2f, 0x35, 0x10, 0x7d
                ]
                .into()
            ))
        )
    }

    #[test]
    fn object_parses_basic_objects() {
        assert_eq!(
            object.parse_peek(b"blob 3\0\x03\x03\x01"),
            Ok((&b""[..], Object::Blob(Blob(vec![0x03, 0x03, 0x01]))))
        );
    }
}
