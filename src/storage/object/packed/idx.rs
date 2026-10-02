//! This module deals with the packfile index format.

use winnow::{
    ModalResult, Parser,
    binary::{be_u32, be_u64},
    combinator::{repeat, seq},
    error::StrContext,
};

use crate::{
    object::hash::{ObjectHash, parse_object_hash},
    parsing::Stream,
};

pub fn header(input: &mut Stream<'_>) -> ModalResult<()> {
    seq!(
        seq!(0xff, 0x74, 0x4f, 0x63).context(StrContext::Label("magic bytes")),
        seq!(0x00, 0x00, 0x00, 0x02).context(StrContext::Label("version"))
    )
    .void()
    .parse_next(input)
}

pub fn fanout_table(input: &mut Stream<'_>) -> ModalResult<Vec<u32>> {
    // TODO: should we do some basic validation here to ensure they are cumulative? (because if
    // they're not, we're probably reading something wrong or the packfile is totally garbage)
    repeat(256, be_u32)
        .context(StrContext::Label("fanout table"))
        .parse_next(input)
}

pub fn object_names(amount: usize) -> impl FnMut(&mut Stream<'_>) -> ModalResult<Vec<ObjectHash>> {
    move |input: &mut Stream<'_>| repeat(amount, parse_object_hash).parse_next(input)
}

pub fn crc_entries(amount: usize) -> impl FnMut(&mut Stream<'_>) -> ModalResult<Vec<u32>> {
    move |input: &mut Stream<'_>| repeat(amount, be_u32).parse_next(input)
}

// significant part about this: each entry here should use up to 31 bytes.
// if it uses more than 31 bytes, i.e. the MSB is 1, then this is not an offset,
// but an index (with the MSB 0) to the
pub fn offsets_1(amount: usize) -> impl FnMut(&mut Stream<'_>) -> ModalResult<Vec<u32>> {
    move |input: &mut Stream<'_>| repeat(amount, be_u32).parse_next(input)
}

// this is the second offsets table. the amount of entries this has depends on the first table. in
// particular it depends on the amount of entries it has that have their MSB set to 1.
pub fn offsets_2(amount: usize) -> impl FnMut(&mut Stream<'_>) -> ModalResult<Vec<u64>> {
    move |input: &mut Stream<'_>| repeat(amount, be_u64).parse_next(input)
}
