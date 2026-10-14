use std::{fmt::Debug, io::Write};

use humansize::{DECIMAL, format_size};
use sha1::{Digest, Sha1};
use winnow::Parser;

use crate::{
    MinigitError,
    error::ParserContext,
    object::{
        Object, ObjectType, blob::parse_blob, commit::parse_commit, hash::ObjectHash,
        tag::parse_tag, tree::parse_tree,
    },
    parsing::Stream,
};

/// An object that has not been parsed yet.
///
/// This struct contains its raw, uncompressed bytes (minus its header), and it is not guaranteed
/// that it is a valid object.
///
/// At any given time it can be turned into a real [Object] using [`Self::into_object`].
#[derive(PartialEq, Eq, Clone)]
pub struct RawObject {
    pub r#type: ObjectType,
    pub bytes: Vec<u8>,
}

impl RawObject {
    /// Creates a new [RawObject] given its type and raw bytes.
    ///
    /// Does **not** perform any validation.
    pub fn new(r#type: ObjectType, bytes: Vec<u8>) -> Self {
        Self { r#type, bytes }
    }

    /// Parses the object bytes.
    pub fn into_object(&self, context: ParserContext) -> Result<Object, MinigitError> {
        let mut stream = Stream::new(&self.bytes);

        match self.r#type {
            ObjectType::Blob => parse_blob.map(Object::from).parse_next(&mut stream),
            ObjectType::Tree => parse_tree.map(Object::from).parse_next(&mut stream),
            ObjectType::Commit => parse_commit.map(Object::from).parse_next(&mut stream),
            ObjectType::Tag => parse_tag.map(Object::from).parse_next(&mut stream),
        }
        // TODO: we are losing context here
        .map_err(|err| MinigitError::ParserError(err.to_string(), context))
    }

    /// Computes the hash of the object.
    pub fn hash(&self) -> Result<ObjectHash, MinigitError> {
        let mut buf = Vec::new();

        write!(
            buf,
            "{} {}\0",
            match self.r#type {
                ObjectType::Blob => "blob",
                ObjectType::Tree => "tree",
                ObjectType::Commit => "commit",
                ObjectType::Tag => "tag",
            },
            self.bytes.len()
        )?;

        buf.write_all(&self.bytes)?;

        Ok(Sha1::digest(buf).into())
    }
}

impl Debug for RawObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RawObject({:?}, {})",
            self.r#type,
            format_size(self.bytes.len(), DECIMAL)
        )
    }
}
