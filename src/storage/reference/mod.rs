//! This module deals with the storage and parsing of refs (`.git/refs`)

use std::{fs, path::Path, str::FromStr};

use winnow::{
    ModalResult, Parser,
    ascii::{newline, till_line_ending},
    combinator::{alt, opt, preceded, terminated},
};

use crate::{
    MinigitError,
    error::ParserContext,
    object::hash::{ObjectHash, parse_object_hash_str},
    parsing::Stream,
};

pub mod db;

pub type RefName = String;

#[derive(Debug, PartialEq, Eq)]
pub enum RefPointer {
    Hash(ObjectHash),
    Symbolic(RefName),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Ref {
    pub name: RefName,
    pub pointer: RefPointer,
    pub peeled: Option<ObjectHash>,
}

impl Ref {
    /// Opens a loose ref given its path, and given its name.
    ///
    /// No validation is done to ensure the name matches the path, so is trusted.
    pub fn open_loose(path: impl AsRef<Path>, name: &RefName) -> Result<Ref, MinigitError> {
        let path = path.as_ref();

        let contents = fs::read(path)?;

        let pointer = parse_ref_pointer
            .parse(Stream::new(&contents))
            .map_err(|err| {
                MinigitError::ParserError(err.to_string(), ParserContext::File(path.to_path_buf()))
            })?;

        Ok(Ref {
            name: name.to_owned(),
            pointer,
            peeled: None,
        })
    }

    pub fn is_peeled(&self) -> bool {
        self.peeled.is_some()
    }
}

impl FromStr for RefPointer {
    type Err = MinigitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_ref_pointer
            .parse(Stream::new(s.as_bytes()))
            .map_err(|err| MinigitError::ParserError(err.to_string(), ParserContext::None))
    }
}

pub fn parse_ref_pointer(input: &mut Stream<'_>) -> ModalResult<RefPointer> {
    terminated(
        alt((
            parse_object_hash_str.map(RefPointer::Hash),
            preceded(
                "ref: ",
                till_line_ending
                    .map(str::from_utf8)
                    .verify_map(Result::ok)
                    .map(str::to_owned),
            )
            .map(RefPointer::Symbolic),
        )),
        opt(newline),
    )
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ref_pointer_hash_with_newline() {
        let input = b"61d5cf2c92d28f042275da58239417db20c5c3da\n";
        let pointer = parse_ref_pointer.parse(Stream::new(input));
        assert!(pointer.is_ok(), "expected Ok, got {:?}", pointer);
    }

    #[test]
    fn parses_ref_pointer_symbolic_without_newline() {
        let input = b"ref: refs/heads/main\n";
        let pointer = parse_ref_pointer.parse(Stream::new(input)).unwrap();
        assert_eq!(pointer, RefPointer::Symbolic("refs/heads/main".into()));
    }
}
