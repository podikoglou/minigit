//! This module deals with the packfile index format.

use winnow::{
    ModalResult, Parser,
    binary::be_u32,
    combinator::{repeat, seq},
    error::StrContext,
};

use crate::parsing::Stream;

fn header(input: &mut Stream<'_>) -> ModalResult<()> {
    seq!(
        seq!(0xff, 0x74, 0x4f, 0x63).context(StrContext::Label("magic bytes")),
        seq!(0x00, 0x00, 0x00, 0x02).context(StrContext::Label("version"))
    )
    .void()
    .parse_next(input)
}

fn fanout_table(input: &mut Stream<'_>) -> ModalResult<Vec<u32>> {
    // TODO: should we do some basic validation here to ensure they are cumulative? (because if
    // they're not, we're probably reading something wrong or the packfile is totally garbage)
    repeat(256, be_u32)
        .context(StrContext::Label("fanout table"))
        .parse_next(input)
}
