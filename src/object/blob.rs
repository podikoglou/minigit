use sha1::{Digest, Sha1};

use crate::hash::{HashObject, ObjectHash};

#[derive(Debug)]
pub struct Blob(Vec<u8>);

impl HashObject for Blob {
    fn hash(&self) -> ObjectHash {
        Sha1::digest(&self.0).into()
    }
}
