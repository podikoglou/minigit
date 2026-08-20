use sha1::digest::{array::Array, consts::U20};

pub trait HashObject {
    /// Creates a SHA1 hash that identifies this object.
    fn hash(&self) -> Array<u8, U20>;
}
