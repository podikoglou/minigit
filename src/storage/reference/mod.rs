//! This module deals with the storage and parsing of refs (`.git/refs`)

use std::{
    fs,
    path::{Component, Path, PathBuf},
    str::FromStr,
};

use bstr::ByteSlice;
use winnow::{
    ModalResult, Parser,
    combinator::{alt, preceded},
    token::rest,
};

use crate::{
    MinigitError,
    error::{self, ParserContext},
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
    path: PathBuf,

    /// The path relative to .git/, like refs/heads/main
    pub name: String,

    pub pointer: RefPointer,
}

impl LazyRef {
    pub fn new(path: PathBuf, pointer: RefPointer) -> Self {
        // figure out the path of the rev relative to .git
        // like refs/heads/main
        //
        // this is of course quite ugly, hopefully there's a better way to do this
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

        Self {
            path,
            name,
            pointer,
        }
    }

    /// Opens and parses (but does not resolve) a ref file.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, MinigitError> {
        let contents = fs::read(&path)?;

        let contents_str = contents.to_str().map_err(|err| {
            MinigitError::ParserError(
                err.to_string(),
                ParserContext::File(path.as_ref().to_path_buf()),
            )
        })?;

        // TODO: how do we pass some ParserContext?
        let pointer = RefPointer::from_str(contents_str.trim())?;

        Ok(LazyRef::new(path.as_ref().to_path_buf(), pointer))
    }
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
