//! This module deals with packfiles
//!
//! References:
//! - <https://git-scm.com/docs/pack-format>
//! - <https://git-scm.com/book/en/v2/Git-Internals-Packfiles>
//!
//! In particular, this deals with version 2 of the packfile format.

mod idx;

use strum::FromRepr;
use winnow::{
    ModalResult, Parser,
    binary::be_u32,
    combinator::seq,
    error::{ContextError, ErrMode},
    token::{any, take},
};

use crate::parsing::Stream;

pub struct PackfileHeader {
    /// The amount of objects contained in the packfile.
    pub objects: u32,
}

fn magic_bytes(input: &mut Stream<'_>) -> ModalResult<()> {
    "PACK".void().parse_next(input)
}

fn version(input: &mut Stream<'_>) -> ModalResult<()> {
    seq!(0x00, 0x00, 0x00, 0x02).void().parse_next(input)
}

fn objects_amount(input: &mut Stream<'_>) -> ModalResult<u32> {
    be_u32.parse_next(input)
}

/// Parses the header of a packfile, returning the amount of objects contained in the header.
fn header(input: &mut Stream<'_>) -> ModalResult<PackfileHeader> {
    seq! {PackfileHeader { _: magic_bytes, _: version, objects: objects_amount }}.parse_next(input)
}

#[derive(FromRepr, Debug, PartialEq)]
#[repr(u8)]
enum PackedObjectType {
    Commit = 1,
    Tree = 2,
    Blob = 3,
    Tag = 4,
    OfsDelta = 6,
    RefDelta = 7,
}

struct PackedObjectHeader {
    r#type: PackedObjectType,
    length: u64,
}

fn object_header(input: &mut Stream<'_>) -> ModalResult<PackedObjectHeader> {
    // TODO: using combinators instead of doing this imperatively would be great.

    let should_continue = |byte: u8| byte & 0b10000000 != 0;

    let mut current_byte = any.parse_next(input)?;

    let type_id = (current_byte & 0b01110000) >> 4;

    // TODO: better error handling
    let r#type =
        PackedObjectType::from_repr(type_id).ok_or_else(|| ErrMode::Cut(ContextError::new()))?;

    let mut size = (current_byte & 0b00001111) as u64;
    let mut pos = 4;

    while should_continue(current_byte) {
        current_byte = any.parse_next(input)?;

        // these are the bits that constitute the size, which is the byte as-is, but with the first
        // bit masked off as it's the continuation bit
        let chunk = (current_byte & 0b0111_1111) as u64;

        size |= chunk << pos;
        pos += 7;
    }

    Ok(PackedObjectHeader {
        r#type,
        length: size,
    })
}

fn undeltified_object(input: &mut Stream<'_>) -> ModalResult<()> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::{PackedObjectType, object_header};
    use winnow::Parser;

    #[test]
    fn object_header_parses_basic_headers() {
        let (rest, header) = object_header
            .parse_peek(&[0x99, 0x0a, 0x78, 0x9c][..])
            .expect("commit header should parse");
        assert_eq!(header.r#type, PackedObjectType::Commit);
        assert_eq!(header.length, 169);
        assert_eq!(rest, &[0x78, 0x9c]);

        let (rest, header) = object_header
            .parse_peek(&[0xb7, 0x02, 0x78, 0x9c][..])
            .expect("blob header should parse");
        assert_eq!(header.r#type, PackedObjectType::Blob);
        assert_eq!(header.length, 39);
        assert_eq!(rest, &[0x78, 0x9c]);

        let (rest, header) = object_header
            .parse_peek(&[0xa4, 0x02, 0x78, 0x9c][..])
            .expect("tree header should parse");
        assert_eq!(header.r#type, PackedObjectType::Tree);
        assert_eq!(header.length, 36);
        assert_eq!(rest, &[0x78, 0x9c]);
    }

    #[test]
    fn object_header_parses_single_byte_header() {
        let (rest, header) = object_header
            .parse_peek(&[0x19, 0x0a][..])
            .expect("commit header should parse");
        assert_eq!(header.r#type, PackedObjectType::Commit);
        assert_eq!(header.length, 9);
        assert_eq!(rest, &[0x0a]);
    }
}
