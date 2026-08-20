use sha1::digest::{array::Array, consts::U20};

/// A hash that identifies an [`crate::object::Object`].
pub struct ObjectHash(Array<u8, U20>);

impl ObjectHash {
    /// Gets the prefix (first two bytes) of the hash.
    pub fn get_prefix(&self) -> &[u8] {
        &self.0[0..2]
    }
}

impl From<Array<u8, U20>> for ObjectHash {
    fn from(value: Array<u8, U20>) -> Self {
        ObjectHash(value)
    }
}

pub trait HashObject {
    /// Creates a SHA1 hash that identifies an [`crate::object::Object`].
    fn hash(&self) -> ObjectHash;
}
