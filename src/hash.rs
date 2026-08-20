use sha1::digest::{array::Array, consts::U20};

/// A hash that identifies an [`crate::object::Object`].
pub type ObjectHash = Array<u8, U20>;

pub trait HashObject {
    /// Creates a SHA1 hash that identifies an [`crate::object::Object`].
    fn hash(&self) -> ObjectHash;
}
