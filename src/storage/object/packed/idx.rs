//! This module deals with the packfile index format.
//!
//! # Maintenance Note
//!
//! Some of the parsers here are the same or similar with the packfile parser,
//! but they should not be merged because there is always the case of them diverging.
//!
//! Further, they serve different purposes and are used in different contexts, so it's
//! good for them to be semantically separated.

use winnow::{ModalResult, Parser, combinator::seq};

use crate::parsing::Stream;

fn magic_bytes(input: &mut Stream<'_>) -> ModalResult<()> {
    seq!(0xff, 0x74, 0x4f, 0x63).void().parse_next(input)
}

fn version(input: &mut Stream<'_>) -> ModalResult<()> {
    seq!(0x00, 0x02).void().parse_next(input)
}

fn header(input: &mut Stream<'_>) -> ModalResult<()> {
    seq!(magic_bytes).void().parse_next(input)
}
