use sha1::{Digest, Sha1};

use crate::hash::{HashObject, ObjectHash};

#[derive(Debug)]
pub struct Tree {}

impl HashObject for Tree {
    fn hash(&self) -> ObjectHash {
        Sha1::digest([]).into()
    }
}
