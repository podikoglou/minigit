use std::io::Write;

/// An blob: an object that simply contains some bytes.
///
/// [trees](`super::Tree`) refer to blobs, usually.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Blob(pub Vec<u8>);

impl Blob {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl Blob {
    /// Writes the blob into a writer.
    ///
    /// This simply writes the raw bytes.
    pub fn write<W: Write>(&self, mut writer: W) -> Result<(), std::io::Error> {
        writer.write_all(&self.0)
    }
}
