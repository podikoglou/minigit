use crate::object::{ObjectType, commit::Identity, hash::ObjectHash};

/// A tag: an object that points to an [crate::object::Object].
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Tag {
    /// The object it's pointing to, along with its type.
    pub target: TagTarget,

    /// The name of the tag.
    pub name: String,

    /// The creator of the tag.
    pub creator: Identity,

    /// The description of the tag.
    pub description: String,
}

impl Tag {
    pub fn new(target: TagTarget, name: String, creator: Identity, description: String) -> Self {
        Self {
            target,
            name,
            creator,
            description,
        }
    }
}

/// The hash and type of an object a tag is pointing to.
pub type TagTarget = (ObjectHash, ObjectType);
