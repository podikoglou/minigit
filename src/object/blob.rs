/// An object which simply contains some bytes. [`super::Tree`] refer to blobs, usually.
#[derive(Debug)]
pub struct Blob(pub Vec<u8>);

impl Blob {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}
