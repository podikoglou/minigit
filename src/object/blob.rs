use std::io::Write;

use crate::writable::Writable;

/// An blob: an object that simply contains some bytes.
///
/// [trees](`super::Tree`) refer to blobs, usually.
#[derive(Debug)]
pub struct Blob(pub Vec<u8>);

impl Blob {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl Writable for Blob {
    /// Writes the blob into a writer.
    ///
    /// This simply writes the raw bytes.
    fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.0)
    }
}
