use std::fmt::Display;

use sha1::{Digest, Sha1};

use crate::hash::{HashObject, ObjectHash};

#[derive(Debug)]
pub struct Tree {}

impl Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl HashObject for Tree {
    fn hash(&self) -> ObjectHash {
        Sha1::digest([]).into()
    }
}
