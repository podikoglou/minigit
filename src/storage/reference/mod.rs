//! This module deals with the storage and parsing of refs (`.git/refs`)

use std::{path::PathBuf, str::FromStr};

use winnow::{
    ModalResult, Parser,
    combinator::{alt, preceded},
    token::rest,
};

use crate::{
    MinigitError, error,
    object::hash::{ObjectHash, parse_object_hash_str},
    parsing::Stream,
};

/// A resolved ref pointing to an object.
#[derive(Debug, PartialEq, Eq)]
pub struct Ref {
    pub path: PathBuf,
    pub pointer: ObjectHash,
}

/// An unresolved ref pointing to an object.
///
/// There are two possible cases for lazy refs:
/// - the `pointer` is already a hash (but still requires resolving to be turned to a [`Ref`])
/// - the `pointer` is the name of another ref, which must be resolved
///
/// These are resolved through [`crate::storage::Store`]
#[derive(Debug, PartialEq, Eq)]
pub struct LazyRef {
    pub path: PathBuf,
    pub pointer: RefPointer,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RefPointer {
    Object(ObjectHash),
    Ref(PathBuf),
}

impl FromStr for RefPointer {
    type Err = MinigitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_ref_pointer
            .parse(Stream::new(s.as_bytes()))
            .map_err(|err| MinigitError::ParserError(err.to_string(), error::ParserContext::None))
    }
}

pub fn parse_ref_pointer(input: &mut Stream<'_>) -> ModalResult<RefPointer> {
    alt((
        parse_object_hash_str.map(RefPointer::Object),
        preceded("ref: ", rest.map(str::from_utf8).verify_map(Result::ok))
            .map(PathBuf::from_str)
            .verify_map(Result::ok)
            .map(RefPointer::Ref),
    ))
    .parse_next(input)
}
