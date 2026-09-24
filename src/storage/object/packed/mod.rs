//! This module deals with [Packfiles](https://git-scm.com/book/en/v2/Git-Internals-Packfiles)

use winnow::{ModalResult, Parser, binary::be_u32, combinator::seq};

use crate::parsing::Stream;

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
fn header(input: &mut Stream<'_>) -> ModalResult<u32> {
    seq!(magic_bytes, version, objects_amount)
        .map(|x| x.2)
        .parse_next(input)
}
