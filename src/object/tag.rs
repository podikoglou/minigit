use std::io::Write;

use winnow::{
    ModalResult, Parser,
    combinator::seq,
    error::StrContext,
    token::{rest, take_until},
};

use crate::{
    identity::{Identity, parse_identity},
    object::{ObjectType, hash::ObjectHash, parse_object_type},
    storage::object::loose::{
        WriteLoose,
        parser::{Stream, object_hash_str, property},
    },
    time::{Timestamp, parse_timestamp},
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

/// Parses a tag object from some bytes.
pub fn parse_tag<'a>(input: &mut Stream<'a>) -> ModalResult<Tag> {
    seq! {Tag{
    target: seq!(
        property("object", object_hash_str),
        property("type", parse_object_type),
    ),
    name: property("tag", take_until(1.., "\n").map(str::from_utf8).verify_map(Result::ok).map(str::to_owned)),
    tagger: property("tagger", seq!(parse_identity, _: " ", parse_timestamp)),
    _: "\n",
    description: rest.map(str::from_utf8).verify_map(Result::ok).map(str::to_owned),
    }}.context(StrContext::Label("tag object"))
    .parse_next(input)
}
