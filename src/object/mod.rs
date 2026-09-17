pub mod blob;
pub mod commit;
pub mod tag;
pub mod tree;

pub mod hash;

use std::io::Write;

use blob::Blob;
use flate2::{Compression, write::ZlibEncoder};
use sha1::{Digest, Sha1};
use strum::{EnumDiscriminants, EnumString, IntoStaticStr, VariantArray};
use tree::Tree;
use winnow::{
    ModalResult, Parser,
    combinator::alt,
    error::{StrContext, StrContextValue},
    token::literal,
};

use crate::{
    MinigitError,
    object::{commit::Commit, hash::ObjectHash, tag::Tag},
    storage::object::{
        LazyObject,
        loose::{WriteLoose, parser::Stream},
    },
};

#[derive(Debug, PartialEq, Eq, Clone, EnumDiscriminants)]
#[strum_discriminants(name(ObjectType))]
#[strum_discriminants(derive(EnumString, IntoStaticStr, VariantArray))]
#[strum_discriminants(strum(ascii_case_insensitive))]
#[strum_discriminants(strum(serialize_all = "lowercase"))]
pub enum Object {
    Blob(Blob),
    Tree(Tree),
    Commit(Box<Commit>),
    Tag(Tag),
}

impl Object {
    /// Creates a SHA1 hash of the object.
    pub fn hash(&self) -> Result<ObjectHash, MinigitError> {
        let mut buf = Vec::new();
        self.write_loose(&mut buf)?;

        Ok(Sha1::digest(buf).into())
    }
}

impl WriteLoose for Object {
    /// Writes the uncompressed object to a write.
    fn write_loose<W: Write>(&self, writer: &mut W) -> Result<(), crate::MinigitError> {
        let mut buf: Vec<u8> = Vec::new();

        match self {
            Object::Blob(blob) => {
                write!(writer, "blob ")?;

                blob.write_loose(&mut buf)?;
            }
            Object::Tree(tree) => {
                write!(writer, "tree ")?;

                tree.write_loose(&mut buf)?;
            }
            Object::Commit(commit) => {
                write!(writer, "commit ")?;

                commit.write_loose(&mut buf)?;
            }
            Object::Tag(tag) => {
                write!(writer, "tag ")?;

                tag.write_loose(&mut buf)?;
            }
        }

        write!(writer, "{}\0", buf.len())?;
        writer.write_all(&buf)?;

        Ok(())
    }
}

impl Object {
    pub fn write_loose_compressed<W: Write>(
        &self,
        writer: &mut W,
    ) -> Result<(), crate::MinigitError> {
        let mut zlib_writer = ZlibEncoder::new(writer, Compression::default());

        self.write_loose(&mut zlib_writer)
    }
}

impl From<Blob> for Object {
    fn from(val: Blob) -> Self {
        Self::Blob(val)
    }
}

impl From<Tree> for Object {
    fn from(val: Tree) -> Self {
        Self::Tree(val)
    }
}

impl From<Commit> for Object {
    fn from(val: Commit) -> Self {
        Self::Commit(Box::new(val))
    }
}
impl From<Tag> for Object {
    fn from(val: Tag) -> Self {
        Self::Tag(val)
    }
}

impl TryFrom<LazyObject> for Object {
    type Error = MinigitError;

    fn try_from(value: LazyObject) -> Result<Self, Self::Error> {
        value.into_object()
    }
}

/// Parses an object type string from some bytes.
pub fn parse_object_type<'a>(input: &mut Stream<'a>) -> ModalResult<ObjectType> {
    alt((
        literal("blob").value(ObjectType::Blob),
        literal("tree").value(ObjectType::Tree),
        literal("commit").value(ObjectType::Commit),
        literal("tag").value(ObjectType::Tag),
    ))
    .context(StrContext::Label("type"))
    .context(StrContext::Expected(StrContextValue::Description(
        "blob | tree | commit | tag",
    )))
    .parse_next(input)
}
