//! This module deals with the packfile index format.

use winnow::{ModalResult, Parser, combinator::seq, error::StrContext};

use crate::parsing::Stream;

fn header(input: &mut Stream<'_>) -> ModalResult<()> {
    seq!(
        seq!(0xff, 0x74, 0x4f, 0x63).context(StrContext::Label("magic bytes")),
        seq!(0x00, 0x00, 0x00, 0x02).context(StrContext::Label("version"))
    )
    .void()
    .parse_next(input)
}
