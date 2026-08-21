pub mod blob;
pub mod tree;

use std::fmt::Display;

use blob::Blob;
use strum::{EnumDiscriminants, EnumString};
use tree::Tree;

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(name(ObjectType))]
#[strum_discriminants(derive(EnumString))]
#[strum_discriminants(strum(ascii_case_insensitive))]
pub enum Object {
    Blob(Blob),
    Tree(Tree),
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Blob(blob) => write!(f, "{}", blob),
            Self::Tree(tree) => write!(f, "{}", tree),
        }
    }
}

impl Object {
    pub fn blob(blob: Blob) -> Self {
        Self::Blob(blob)
    }

    pub fn tree(tree: Tree) -> Self {
        Self::Tree(tree)
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
