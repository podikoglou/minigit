use std::io::Write;

use crate::writable::Writable;

/// A tree: an object that associates file names to [blobs](`super::Blob`) and other trees.
#[derive(Debug)]
pub struct Tree {}

impl Writable for Tree {
    /// Writes the tree into a writer.
    ///
    /// This simply writes the raw bytes.
    fn write<W: Write>(&self, _writer: &mut W) -> std::io::Result<()> {
        todo!()
    }
}
