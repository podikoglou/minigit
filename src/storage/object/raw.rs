use winnow::Parser;

use crate::{
    MinigitError,
    error::ParserContext,
    object::{
        Object, ObjectType, blob::parse_blob, commit::parse_commit, tag::parse_tag,
        tree::parse_tree,
    },
    parsing::Stream,
};

/// An object that has not been parsed yet.
///
/// This struct contains its raw, uncompressed bytes (minus its header), and it is not guaranteed
/// that it is a valid object.
///
/// At any given time it can be turned into a real [Object] using [`Self::into_object`].
#[derive(Debug, PartialEq, Eq, Clone)]
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
    pub fn into_object(&self) -> Result<Object, MinigitError> {
        let mut stream = Stream::new(&self.bytes);

        match self.r#type {
            ObjectType::Blob => parse_blob.map(Object::Blob).parse_next(&mut stream),
            ObjectType::Tree => parse_tree.map(Object::Tree).parse_next(&mut stream),
            ObjectType::Commit => parse_commit.map(Object::from).parse_next(&mut stream),
            ObjectType::Tag => parse_tag.map(Object::from).parse_next(&mut stream),
        }
        // TODO: we are losing context here
        .map_err(|err| MinigitError::ParserError(err.to_string(), ParserContext::None))
    }
}
