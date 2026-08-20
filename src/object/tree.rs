use sha1::{
    Digest, Sha1,
    digest::{array::Array, consts::U20},
};

use crate::hash::HashObject;

#[derive(Debug)]
pub struct Tree {}

impl HashObject for Tree {
    fn hash(&self) -> Array<u8, U20> {
        Sha1::digest([])
    }
}
