pub mod blob;
pub mod commit;
pub mod tree;

pub mod hash;

use std::io::{self, Write};

use blob::Blob;
use flate2::{Compression, write::ZlibEncoder};
use sha1::{Digest, Sha1};
use strum::{EnumDiscriminants, EnumString};
use tree::Tree;

use crate::{
    MinigitError,
    object::{commit::Commit, hash::ObjectHash},
    storage::object::{
        LazyObject,
        loose::{self, WriteLoose},
    },
};

#[derive(Debug, PartialEq, Eq, Clone, EnumDiscriminants)]
#[strum_discriminants(name(ObjectType))]
#[strum_discriminants(derive(EnumString))]
#[strum_discriminants(strum(ascii_case_insensitive))]
pub enum Object {
    Blob(Blob),
    Tree(Tree),
    Commit(Commit),
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
        Self::Commit(val)
    }
}

impl TryFrom<LazyObject> for Object {
    type Error = MinigitError;

    fn try_from(value: LazyObject) -> Result<Self, Self::Error> {
        value.into_object()
    }
}
