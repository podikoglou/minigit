use std::io::Write;

use crate::{
    identity::Identity,
    object::{ObjectType, hash::ObjectHash},
    storage::object::loose::WriteLoose,
    time::Timestamp,
};

/// A tag: an object that points to an [crate::object::Object].
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Tag {
    /// The object it's pointing to, along with its type.
    pub target: TagTarget,

    /// The name of the tag.
    pub name: String,

    /// The tagger and time of creation of the tag.
    pub tagger: (Identity, Timestamp),

    /// The description of the tag.
    pub description: String,
}

impl Tag {
    pub fn new(
        target: TagTarget,
        name: String,
        tagger: (Identity, Timestamp),
        description: String,
    ) -> Self {
        Self {
            target,
            name,
            tagger,
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
        writeln!(writer, "tag {}", self.name)?;
        write!(writer, "tagger ")?;
        self.tagger.write_loose(writer)?;
        writeln!(writer)?;

        writeln!(writer)?;
        write!(writer, "{}", self.description)?;

        Ok(())
    }
}
