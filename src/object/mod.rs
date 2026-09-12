pub mod blob;
pub mod tree;

pub mod hash;

use std::io::{self, Write};

use blob::Blob;
use sha1::{Digest, Sha1};
use strum::{EnumDiscriminants, EnumString};
use tree::Tree;

use crate::{object::hash::ObjectHash, storage::object::LazyObject};

#[derive(Debug, PartialEq, Eq, EnumDiscriminants)]
#[strum_discriminants(name(ObjectType))]
#[strum_discriminants(derive(EnumString))]
#[strum_discriminants(strum(ascii_case_insensitive))]
pub enum Object {
    Blob(Blob),
    Tree(Tree),
}

impl Object {
    /// Writes the header of the object, to a [`Write`].
    pub fn write_header<W: Write>(&self, mut writer: W) -> Result<(), std::io::Error> {
        let (tag, size) = match self {
            Object::Blob(blob) => ("blob", blob.0.len()),
            Object::Tree(_) => todo!(),
        };

        write!(writer, "{} {}\0", tag, size)
    }

    /// Writes the uncompressed object, including its header, to a [`Write`].
    pub fn write<W: Write>(&self, mut writer: W) -> Result<(), std::io::Error> {
        self.write_header(&mut writer)?;

        match self {
            Object::Blob(blob) => blob.write(writer),
            Object::Tree(tree) => tree.write(writer),
        }
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
        Self::Blob(val)
    }
}

impl From<Tree> for Object {
    fn from(val: Tree) -> Self {
        Self::Tree(val)
    }
}

impl TryFrom<LazyObject> for Object {
    type Error = io::Error;

    fn try_from(value: LazyObject) -> Result<Self, io::Error> {
        Ok(todo!())
    }
}
