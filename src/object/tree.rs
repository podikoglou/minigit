use std::io::Write;

/// A tree: an object that associates file names to [blobs](`super::Blob`) and other trees.
#[derive(Debug)]
pub struct Tree {}

impl Tree {
    /// Writes the tree into a writer.
    ///
    /// This simply writes the raw bytes.
    pub fn write<W: Write>(&self, mut _writer: W) -> Result<(), std::io::Error> {
        todo!()
    }
}
