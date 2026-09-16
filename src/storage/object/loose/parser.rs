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
    ascii::{dec_uint, digit1, oct_digit1, till_line_ending},
    combinator::{alt, repeat, seq, terminated},
    error::{ContextError, ErrMode, StrContext, StrContextValue},
    token::{literal, rest, take, take_till, take_until},
};

use crate::{
    MinigitError,
    error::ParserContext,
    fs::{FileName, parse_file_name},
    object::{
        Object, ObjectType,
        blob::Blob,
        commit::{Commit, CommitProperty, Identity},
        hash::ObjectHash,
        tag::Tag,
        tree::{Tree, TreeEntry},
    },
};

type Stream<'a> = &'a [u8];

/// Parses an [Object] from some bytes.
///
/// Unless you're building your own parsers this is the function you're looking for.
pub fn parse_object(input: &[u8], context: ParserContext) -> Result<Object, MinigitError> {
    object
        .parse(input)
        .map_err(|err| MinigitError::ParserError(err.to_string(), context))
}

/// Parses an [Object] from some input.
pub fn object<'a>(input: &mut Stream<'a>) -> ModalResult<Object> {
    let (typee, size) = header.parse_next(input)?;
    let mut bytes: Stream<'a> = take(size).parse_next(input)?;

    match typee {
        ObjectType::Blob => blob.map(Object::Blob).parse_next(&mut bytes),
        ObjectType::Tree => tree.map(Object::Tree).parse_next(&mut bytes),
        ObjectType::Commit => commit.map(Object::from).parse_next(&mut bytes),
        ObjectType::Tag => tag.map(Object::from).parse_next(&mut bytes),
    }
}

/// Parse a header (object type and size) from some bytes.
pub fn header<'a>(input: &mut Stream<'a>) -> ModalResult<(ObjectType, usize)> {
    let mut size = dec_uint::<_, usize, ErrMode<ContextError>>
        .context(StrContext::Label("payload size"))
        .context(StrContext::Expected(StrContextValue::Description(
            "bytes amount",
        )));

    seq!(object_type, _: " ", size, _: "\0")
        .context(StrContext::Label("header"))
        .parse_next(input)
}

/// Parses an object type string from some bytes.
pub fn object_type<'a>(input: &mut Stream<'a>) -> ModalResult<ObjectType> {
    alt((
        literal("blob").value(ObjectType::Blob),
        literal("tree").value(ObjectType::Tree),
        literal("commit").value(ObjectType::Commit),
        literal("tag").value(ObjectType::Tag),
    ))
    .context(StrContext::Label("type"))
    .context(StrContext::Expected(StrContextValue::Description(
        "blob | tree | commit | tag",
    )))
    .parse_next(input)
}

/// Parses a blob object's content from some bytes.
pub fn blob<'a>(input: &mut Stream<'a>) -> ModalResult<Blob> {
    rest.map(|e: Stream| Blob(e.into()))
        .context(StrContext::Label("blob object"))
        .parse_next(input)
}

/// Parses a tree object from some bytes.
pub fn tree<'a>(input: &mut Stream<'a>) -> ModalResult<Tree> {
    // NOTE: not sure if this should be `0..` or `1..`
    // should we be able to parse empty trees?
    repeat(0.., tree_entry)
        .map(|entries: Vec<(u16, FileName, ObjectHash)>| {
            entries
                .into_iter()
                .map(|(mode, name, hash)| (name, TreeEntry::new(mode, hash)))
                .collect::<BTreeMap<FileName, TreeEntry>>()
        })
        .context(StrContext::Label("tree object"))
        .map(Tree::new)
        .parse_next(input)
}

/// Helper for creating parsers that parse a key value pair found in a commit object, such as
/// `author <author>`
fn property<'a, O>(
    mut key: impl Parser<Stream<'a>, &'a [u8], ErrMode<ContextError>>,
    mut value: impl Parser<Stream<'a>, O, ErrMode<ContextError>>,
) -> impl Parser<Stream<'a>, O, ErrMode<ContextError>> {
    seq!(_: key, _: " ", value, _: "\n").map(|(value,)| value)
}

/// Helper for creating parsers that parse a multi-line key value pair found in a commit object,
/// such as `gpgsig`, where continuation lines start with a single space.
pub fn multiline_property<'a>(
    mut key: impl Parser<Stream<'a>, &'a [u8], ErrMode<ContextError>>,
) -> impl Parser<Stream<'a>, String, ErrMode<ContextError>> {
    seq!(
        _: key,
        _: " ",
        terminated(take_until(0.., "\n"), "\n")
            .map(str::from_utf8)
            .verify_map(Result::ok),
        repeat(
            0..,
            seq!(_: " ", terminated(take_until(0.., "\n"), "\n"))
                .map(|(line,)| line)
                .map(str::from_utf8)
                .verify_map(Result::ok),
        ),
    )
    .map(|(first_line, continuation_lines): (&str, Vec<&str>)| {
        let mut lines = Vec::with_capacity(1 + continuation_lines.len());
        lines.push(first_line);
        lines.extend(continuation_lines);
        lines.join("\n")
    })
}

/// Parses an arbitrary commit property including its name and value.
pub fn extra_property<'a>(input: &mut Stream<'a>) -> ModalResult<CommitProperty> {
    seq!(
        // name
        take_till(1.., (b' ', b'\n'))
            .map(str::from_utf8)
            .verify_map(Result::ok),

        // separator
        _: " ",

        // value
        seq!(
            terminated(take_until(0.., "\n"), "\n")
                .map(str::from_utf8)
                .verify_map(Result::ok),
            repeat(
                0..,
                seq!(_: " ", terminated(take_until(0.., "\n"), "\n"))
                    .map(|(line,)| line)
                    .map(str::from_utf8)
                    .verify_map(Result::ok),
            )
        )
        .map(|(first_line, continuation_lines): (&str, Vec<&str>)| {
            let mut lines = Vec::with_capacity(1 + continuation_lines.len());
            lines.push(first_line);
            lines.extend(continuation_lines);
            lines.join("\n")
        })
    )
    .map(|(key, value)| (key.to_owned(), value))
    .parse_next(input)
}

/// Parses a commit object from some bytes.
pub fn commit<'a>(input: &mut Stream<'a>) -> ModalResult<Commit> {
    seq! {Commit{
        tree: property("tree", object_hash_str),
        parents: repeat(0.., property("parent", object_hash_str)),
        author: property("author", seq!(identity, _: " ", timestamp)),
        committer: property("committer", seq!(identity, _: " ", timestamp)),
        extra: repeat(0.., extra_property),
        _: "\n",
        description: rest.map(str::from_utf8).verify_map(Result::ok).map(str::to_owned),
    }}
    .context(StrContext::Label("commit object"))
    .parse_next(input)
}
/// Parses a tag object from some bytes.
pub fn tag<'a>(input: &mut Stream<'a>) -> ModalResult<Tag> {
    seq! {Tag{
    target: seq!(
        property("object", object_hash_str),
        property("type", object_type),
    ),
    name: property("tag", till_line_ending.map(str::from_utf8).verify_map(Result::ok).map(str::to_owned)),
    tagger: property("tagger", seq!(identity, _: " ", timestamp)),
    _: "\n",
    description: rest.map(str::from_utf8).verify_map(Result::ok).map(str::to_owned),
    }}.context(StrContext::Label("tag object"))
    .parse_next(input)
}

/// Parses a tree object's entry into a tuple `(mode, name, hash)` from some bytes.
pub fn tree_entry<'a>(input: &mut Stream<'a>) -> ModalResult<(u16, FileName, ObjectHash)> {
    seq!((mode, _: " ", parse_file_name, object_hash))
        .context(StrContext::Label("tree entry"))
        .parse_next(input)
}

/// Parses a UTF-8 encoded file mode such as 100644 from some bytes.
pub fn mode<'a>(input: &mut Stream<'a>) -> ModalResult<u16> {
    oct_digit1
        .map(str::from_utf8)
        .verify_map(Result::ok)
        .map(|str| u16::from_str_radix(str, 8))
        .verify_map(Result::ok)
        .context(StrContext::Label("file mode"))
        .parse_next(input)
}

/// Parses a binary hash from some bytes.
///
/// To parse a UTF-8 hash, see [object_hash_str].
pub fn object_hash<'a>(input: &mut Stream<'a>) -> ModalResult<ObjectHash> {
    take(20usize)
        .map(Array::try_from)
        .verify_map(Result::ok)
        .map(ObjectHash::from)
        .context(StrContext::Label("object hash"))
        .context(StrContext::Expected(StrContextValue::Description(
            "hash bytes",
        )))
        .parse_next(input)
}

/// Parses a UTF-8 encoded hash from some bytes.
///
/// To parse a binary-encoded hash, see [object_hash].
pub fn object_hash_str<'a>(input: &mut Stream<'a>) -> ModalResult<ObjectHash> {
    take(40usize)
        .map(str::from_utf8)
        .verify_map(Result::ok)
        .map(str::parse::<ObjectHash>)
        .verify_map(Result::ok)
        .context(StrContext::Label("object hash"))
        .context(StrContext::Expected(StrContextValue::Description(
            "hash string",
        )))
        .parse_next(input)
}

/// Parses an identity in the form of `John Doe <john@doe.com>` from some bytes.
pub fn identity<'a>(input: &mut Stream<'a>) -> ModalResult<Identity> {
    seq! {Identity{
        name: take_until(1.., " <").map(str::from_utf8).verify_map(Result::ok).map(str::to_owned).context(StrContext::Label("name")),
        _: " <",
        email: take_until(0.., ">").map(str::from_utf8).verify_map(Result::ok).map(str::to_owned).context(StrContext::Label("email")),
        _: ">",
    }}
    .context(StrContext::Label("identity"))
    .context(StrContext::Expected(StrContextValue::Description(
        "<name> <<email>>",
    )))
    .parse_next(input)
}

/// Parses a [DateTime<FixedOffset>] from some bytes.
pub fn timestamp<'a>(input: &mut Stream<'a>) -> ModalResult<DateTime<FixedOffset>> {
    seq!(
        digit1.parse_to::<i64>(),
        _: " ",
        alt((b'+'.value(1), b'-'.value(-1))),
        take(2usize).parse_to::<i32>(),
        take(2usize).parse_to::<i32>(),
    )
    .verify_map(|(secs, sign, hours, minutes)| {
        let offset = FixedOffset::east_opt(sign * (hours * 3600 + minutes * 60))?;

        DateTime::from_timestamp(secs, 0).map(|dt| dt.with_timezone(&offset))
    })
    .context(StrContext::Label("timestamp"))
    .context(StrContext::Expected(StrContextValue::Description(
        "<unix time> <offset>",
    )))
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use crate::{
        object::{ObjectType, commit::Identity},
        storage::object::loose::parser::{
            extra_property, header, identity, mode, multiline_property, object_hash_str,
            object_type, timestamp, tree_entry,
        },
    };
    use chrono::{DateTime, FixedOffset, NaiveDateTime};
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
    fn tree_entry_parses_valid_entries() {
        assert_eq!(
            tree_entry.parse_peek(b"100644 cli.rs\0\x29\xf3\x23\xb3\x1a\xd1\x29\x96\x4f\xfb\x4f\x97\xf2\x03\xbe\x9c\x2f\x35\x10\x7d"),
            Ok((&b""[..], (0o100644, "cli.rs\0".parse().unwrap(), [0x29, 0xf3, 0x23, 0xb3, 0x1a, 0xd1, 0x29, 0x96, 0x4f, 0xfb, 0x4f, 0x97, 0xf2, 0x03, 0xbe, 0x9c, 0x2f, 0x35, 0x10, 0x7d].into() )))
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
    #[ignore]
    fn identity_parses_valid_identities() {
        assert_eq!(
            identity.parse_peek(b"author John Doe <john@doe.com>"),
            Ok((
                &b""[..],
                Identity::new("John Doe".to_string(), "john@doe.com".to_string())
            ))
        );
    }

    #[test]
    fn identity_rejects_invalid_input() {
        assert_matches!(identity.parse_peek(b"<john@doe.com>"), Err(_));
        assert_matches!(identity.parse_peek(b"j<john@doe.com>"), Err(_));
        assert_matches!(identity.parse_peek(b"<john@doe.com"), Err(_));
        assert_matches!(identity.parse_peek(b"john@doe.com>"), Err(_));
        assert_matches!(identity.parse_peek(b"john@doe.com"), Err(_));
        // TODO: should this validate emails?
        assert_matches!(identity.parse_peek(b"johndoe.com"), Err(_));
    }

    #[test]
    fn timestamp_parses_basic_timestamps() {
        assert_eq!(
            timestamp.parse_peek(b"1789057194 +0300"),
            Ok((
                &b""[..],
                DateTime::<FixedOffset>::from_naive_utc_and_offset(
                    #[allow(deprecated)]
                    NaiveDateTime::from_timestamp(1789057194, 0),
                    FixedOffset::east_opt(3 * 3600).unwrap(),
                )
            ))
        );
    }

    #[test]
    fn multiline_property_parses_valid_signature() {
        let input = b"gpgsig -----BEGIN PGP SIGNATURE-----\n \n wsFcBAABCAAQBQJqc1jACRC1aQ7uu5UhlAAAFfgQACyD2HIkYM5SeaWNsgzpZsVu\n -----END PGP SIGNATURE-----\n \n";
        assert_eq!(
            multiline_property("gpgsig").parse_peek(input),
            Ok((
                &b""[..],
                "-----BEGIN PGP SIGNATURE-----\n\nwsFcBAABCAAQBQJqc1jACRC1aQ7uu5UhlAAAFfgQACyD2HIkYM5SeaWNsgzpZsVu\n-----END PGP SIGNATURE-----\n".to_string()
            ))
        );
    }

    #[test]
    fn extra_property_parses_single_line_property() {
        let input = b"change-id xnxouqnvmpzvuvkotwynowookslovtno\n\nmessage";
        assert_eq!(
            extra_property.parse_peek(input),
            Ok((
                &b"\nmessage"[..],
                (
                    "change-id".to_string(),
                    "xnxouqnvmpzvuvkotwynowookslovtno".to_string()
                )
            ))
        );
    }

    #[test]
    fn extra_property_parses_multi_line_property() {
        let input = b"gpgsig -----BEGIN PGP SIGNATURE-----\n \n wsFcBAABCAAQBQJqc1jACRC1aQ7uu5UhlAAAFfgQACyD2HIkYM5SeaWNsgzpZsVu\n -----END PGP SIGNATURE-----\n \n";
        assert_eq!(
            extra_property.parse_peek(input),
            Ok((
                &b""[..],
                (
                    "gpgsig".to_string(), 
                    "-----BEGIN PGP SIGNATURE-----\n\nwsFcBAABCAAQBQJqc1jACRC1aQ7uu5UhlAAAFfgQACyD2HIkYM5SeaWNsgzpZsVu\n-----END PGP SIGNATURE-----\n".to_string()
                )
            ))
        );
    }

    #[test]
    fn extra_property_rejects_empty_line() {
        assert_matches!(extra_property.parse_peek(b"\n"), Err(ErrMode::Backtrack(_)));
        assert_matches!(
            extra_property.parse_peek(b"\ncommit message\n"),
            Err(ErrMode::Backtrack(_))
        );
    }
}
