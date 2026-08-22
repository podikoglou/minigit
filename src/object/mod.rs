pub mod blob;
pub mod tree;

use std::io::{self, Write};

use blob::Blob;
use sha1::{Digest, Sha1};
use strum::{EnumDiscriminants, EnumString};
use tree::Tree;

use crate::{hash::ObjectHash, storage::object::LazyObject};

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(name(ObjectType))]
#[strum_discriminants(derive(EnumString))]
#[strum_discriminants(strum(ascii_case_insensitive))]
pub enum Object {
    Blob(Blob),
    Tree(Tree),
}

impl Object {
    pub fn blob(blob: Blob) -> Self {
        Self::Blob(blob)
    }

    pub fn tree(tree: Tree) -> Self {
        Self::Tree(tree)
    }

    /// Writes the header of the object, depending on the kind of object, to a [`Write`].
    pub fn write_header<W: Write>(&self, mut writer: W) -> Result<(), std::io::Error> {
        match self {
            Object::Blob(blob) => {
                write!(writer, "blob {}\0", blob.0.len())
            }
            Object::Tree(_) => todo!(),
        }?;

        Ok(())
    }

    /// Writes the uncompressed object, including its header, to a [`Write`].
    pub fn write<W: Write>(&self, mut writer: W) -> Result<(), std::io::Error> {
        self.write_header(&mut writer)?;

        match self {
            Object::Blob(blob) => writer.write(&blob.0)?,
            Object::Tree(_) => todo!(),
        };

        Ok(())
    }

    /// Creates a SHA1 hash of the object.
    pub fn hash(&self) -> Result<ObjectHash, io::Error> {
        let mut buf = Vec::new();
        self.write(&mut buf)?;

        Ok(Sha1::digest(buf).into())
    }
}

impl From<Blob> for Object {
    fn from(val: Blob) -> Self {
        Self::blob(val)
    }
}

impl From<Tree> for Object {
    fn from(val: Tree) -> Self {
        Self::tree(val)
    }
}

impl TryFrom<LazyObject> for Object {
    type Error = io::Error;

    fn try_from(value: LazyObject) -> Result<Self, io::Error> {
        Ok(todo!())
    }
}
