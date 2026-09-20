//! This module deals with the validation of timestamps.
//!
//! It offers the [`Timestamp`] newtime, which models timestamps which are valid in Git.

use chrono::{DateTime, FixedOffset};
use nutype::nutype;
use winnow::{
    ModalResult, Parser,
    ascii::digit1,
    combinator::{alt, seq},
    error::{StrContext, StrContextValue},
    token::take,
};

use crate::parsing::Stream;

#[nutype(
    derive(Debug, PartialEq, Eq, Clone, Display, AsRef, Deref),
    validate(predicate = |datetime| datetime.offset().local_minus_utc() % 60 == 0 && datetime.timestamp_subsec_nanos() == 0 ),
)]
pub struct Timestamp(DateTime<FixedOffset>);

/// Parses a [Timestamp] from some bytes.
pub fn parse_timestamp(input: &mut Stream<'_>) -> ModalResult<Timestamp> {
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
    .map(Timestamp::try_new)
    .verify_map(Result::ok)
    .context(StrContext::Label("timestamp"))
    .context(StrContext::Expected(StrContextValue::Description(
        "<unix time> <offset>",
    )))
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, FixedOffset, NaiveDateTime};
    use winnow::Parser;

    use crate::time::{Timestamp, parse_timestamp};

    #[test]
    fn timestamp_parses_basic_timestamps() {
        assert_eq!(
            parse_timestamp.parse_peek(b"1789057194 +0300"),
            Ok((
                &b""[..],
                Timestamp::try_new(DateTime::<FixedOffset>::from_naive_utc_and_offset(
                    #[allow(deprecated)]
                    NaiveDateTime::from_timestamp(1789057194, 0),
                    FixedOffset::east_opt(3 * 3600).unwrap(),
                ))
                .unwrap()
            ))
        );
    }
}
