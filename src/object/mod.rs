pub mod blob;
pub mod tree;

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

impl Object {
    pub fn blob(blob: Blob) -> Object {
        Object::Blob(blob)
    }

    pub fn tree(tree: Tree) -> Object {
        Object::Tree(tree)
    }
}

impl From<Blob> for Object {
    fn from(val: Blob) -> Self {
        Object::blob(val)
    }
}

impl From<Tree> for Object {
    fn from(val: Tree) -> Self {
        Object::tree(val)
    }
}
