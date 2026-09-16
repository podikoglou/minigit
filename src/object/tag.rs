use std::io::Write;

use chrono::{DateTime, FixedOffset};

use crate::{
    object::{ObjectType, commit::Identity, hash::ObjectHash},
    storage::object::loose::WriteLoose,
};

/// A tag: an object that points to an [crate::object::Object].
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Tag {
    /// The object it's pointing to, along with its type.
    pub target: TagTarget,

    /// The name of the tag.
    pub name: String,

    /// The creator and time of creation of the tag.
    pub creator: (Identity, DateTime<FixedOffset>),

    /// The description of the tag.
    pub description: String,
}

impl Tag {
    pub fn new(
        target: TagTarget,
        name: String,
        creator: (Identity, DateTime<FixedOffset>),
        description: String,
    ) -> Self {
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

impl WriteLoose for Tag {
    fn write_loose<W: Write>(&self, writer: &mut W) -> Result<(), crate::MinigitError> {
        writeln!(writer, "object {}", self.target.0)?;
        writeln!(writer, "type {}", Into::<&'static str>::into(self.target.1))?;
        self.creator.write_loose(writer)?;
        writeln!(writer)?;

        writeln!(writer, "{}", self.description)?;

        Ok(())
    }
}
