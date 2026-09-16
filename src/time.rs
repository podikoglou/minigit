//! This module deals with the validation of timestamps.
//!
//! It offers the [`Timestamp`] newtime, which models timestamps which are valid in Git.

use chrono::{DateTime, FixedOffset};
use nutype::nutype;

#[nutype(
    derive(Debug, PartialEq, Eq, Clone, Display, AsRef, Deref),
    validate(predicate = |datetime| datetime.offset().local_minus_utc() % 60 == 0 && datetime.timestamp_subsec_nanos() == 0 ),
)]
pub struct Timestamp(DateTime<FixedOffset>);
