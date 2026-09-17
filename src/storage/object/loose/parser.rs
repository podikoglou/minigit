//! This module deals with parsing loose object files. The most important function is [`parse_object`].
//!
//! This module contains several incremental parsers built using the `winnow` parser combinator
//! crate. It should be stressed that they will not fail if they have excess input, as they are
//! incremental and built to be combined.

use sha1::digest::array::Array;
use winnow::{
    ModalResult, Parser,
    ascii::oct_digit1,
    combinator::{repeat, seq, terminated},
    error::{ContextError, ErrMode, StrContext, StrContextValue},
    token::{take, take_till, take_until},
};

use crate::object::{commit::CommitProperty, hash::ObjectHash};

pub type Stream<'a> = &'a [u8];

/// Helper for creating parsers that parse a key value pair found in a commit object, such as
/// `author <author>`
pub fn property<'a, O>(
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

#[cfg(test)]
mod tests {
    use crate::storage::object::loose::parser::{
        extra_property, mode, multiline_property, object_hash_str,
    };
    use std::assert_matches;
    use winnow::{Parser, error::ErrMode};

    #[test]
    fn mode_parses_valid_modes() {
        assert_eq!(mode.parse_peek(b"000000"), Ok((&b""[..], 0)));
        assert_eq!(mode.parse_peek(b"100644"), Ok((&b""[..], 0o100644)));
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
