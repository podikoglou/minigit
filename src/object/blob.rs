use sha1::{
    Digest, Sha1,
    digest::{array::Array, consts::U20},
};

use crate::hash::HashObject;

#[derive(Debug)]
pub struct Blob(Vec<u8>);

impl HashObject for Blob {
    fn hash(&self) -> Array<u8, U20> {
        Sha1::digest(&self.0)
    }
}
