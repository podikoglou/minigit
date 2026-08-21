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
