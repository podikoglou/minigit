use crate::object::ObjectType;

/// An object that has not been parsed yet.
///
/// This struct contains its raw, uncompressed bytes (minus its header), and it is not guaranteed
/// that it is a valid object.
///
/// At any given time it can be turned into a real [Object] using [`Self::into_object`].
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RawObject {
    r#type: ObjectType,
    bytes: Vec<u8>,
}

impl RawObject {
    /// Creates a new [RawObject] given its type and raw bytes.
    ///
    /// Does **not** perform any validation.
    pub fn new(r#type: ObjectType, bytes: Vec<u8>) -> Self {
        Self { r#type, bytes }
    }
}
