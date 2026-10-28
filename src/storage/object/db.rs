use crate::{MinigitError, object::hash::ObjectHash, storage::object::RawObject};

/// A database or source that can read raw Git objects by their [`ObjectHash`].
pub trait ObjectDb {
    /// Gets a [`RawObject`] by its [`ObjectHash`].
    fn raw_object(&self, hash: &ObjectHash) -> Result<RawObject, MinigitError>;
}
