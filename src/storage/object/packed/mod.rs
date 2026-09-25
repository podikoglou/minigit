//! This module deals with packfiles
//!
//! References:
//! - <https://git-scm.com/docs/pack-format>
//! - <https://git-scm.com/book/en/v2/Git-Internals-Packfiles>
//!
//! In particular, this deals with version 2 of the packfile format.

mod idx;

use winnow::{ModalResult, Parser, binary::be_u32, combinator::seq};

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
