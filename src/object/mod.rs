pub mod blob;
pub mod tree;

use blob::Blob;
use tree::Tree;

#[derive(Debug)]
pub enum Object {
    Blob(Blob),
    Tree(Tree),
}
