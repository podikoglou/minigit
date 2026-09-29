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
    seq!(0x00, 0x02).void().parse_next(input)
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

    let should_continue = |byte: u8| (byte & 0b10000000 >> 7) == 1;

    let mut current_byte = any.parse_next(input)?;

    let type_id = (current_byte & 0b01110000) >> 4;

    // TODO: better error handling
    let r#type =
        PackedObjectType::from_repr(type_id).ok_or_else(|| ErrMode::Cut(ContextError::new()))?;

    let mut size = (current_byte as u64) << 60;
    let mut pos = 4;

    while should_continue(current_byte) {
        current_byte = any.parse_next(input)?;

        // these are the bits that constitute the size, which is the byte as-is, but with the first
        // bit masked off as it's the continuation bit
        let chunk = (current_byte & 0b0111_1111) as u64;

        size |= chunk << pos;
        pos += 7;
    }

    todo!()

    // any.verify_map(|byte: u8| {
    //     let cont = byte & 0b10000000 == 1;
    //     let type_id = byte & 0b01110000;
    //     let first_nibble = byte & 0b00001111;
    //
    //     match PackedObjectType::from_repr(type_id) {
    //         Some(r#type) => Some((cont, r#type)),
    //         None => None,
    //     }
    // })
}

fn undeltified_object(input: &mut Stream<'_>) -> ModalResult<()> {
    todo!()
}
