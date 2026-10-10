/// An object that has not been parsed yet.
///
/// This struct contains its raw, uncompressed bytes, and it is not guaranteed that it is a valid
/// object.
///
/// At any given time it can be turned into a real [Object] using [`Self::into_object`].
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RawObject(Vec<u8>);

impl RawObject {
    /// Creates a new [RawObject].
    ///
    /// Does **not** perform any validation.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}
