use std::fmt::Display;

use sha1::{Digest, Sha1};

use crate::hash::{HashObject, ObjectHash};

#[derive(Debug)]
pub struct Blob(Vec<u8>);

impl Display for Blob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.hash())
    }
}
impl HashObject for Blob {
    fn hash(&self) -> ObjectHash {
        Sha1::digest(&self.0).into()
    }
}
