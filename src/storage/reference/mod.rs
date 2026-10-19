//! This module deals with the storage and parsing of refs (`.git/refs`)

use std::{
    fs,
    path::{Component, Path, PathBuf},
    str::FromStr,
};

use bstr::ByteSlice;
use winnow::{
    ModalResult, Parser,
    ascii::newline,
    combinator::{alt, preceded, terminated},
    token::rest,
};

use crate::{
    MinigitError,
    error::{self},
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
    pub fn open_loose(path: impl AsRef<Path>) -> Result<Ref, MinigitError> {
        let path = path.as_ref();

        let contents = fs::read(path)?;

        let pointer = parse_loose_ref_file
            .parse(Stream::new(&contents))
            .map_err(|err| {
                MinigitError::ParserError(err.to_string(), error::ParserContext::None)
            })?;

        // extract name like .refs/heads/main or HEAD
        let components: Vec<Component> = path
            .components()
            .rev()
            .take_while(|x| x.as_os_str() != ".git")
            .collect();

        let name: String = components
            .into_iter()
            .rev()
            .map(|x| x.as_os_str())
            .map(|x| x.to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");

        Ok(Ref {
            name,
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
            .map_err(|err| MinigitError::ParserError(err.to_string(), error::ParserContext::None))
    }
}

pub fn parse_loose_ref_file(input: &mut Stream<'_>) -> ModalResult<RefPointer> {
    terminated(parse_ref_pointer, newline).parse_next(input)
}

pub fn parse_ref_pointer(input: &mut Stream<'_>) -> ModalResult<RefPointer> {
    alt((
        parse_object_hash_str.map(RefPointer::Hash),
        preceded(
            "ref: ",
            rest.map(str::from_utf8)
                .verify_map(Result::ok)
                .map(str::to_owned),
        )
        .map(RefPointer::Symbolic),
    ))
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
