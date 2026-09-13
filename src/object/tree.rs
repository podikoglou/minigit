use std::{collections::BTreeMap, io::Write};

use crate::object::hash::ObjectHash;

/// A tree: an object that associates file names to [tree entries](TreeEntry).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Tree {
    /// A mapping from [String] -> [TreeEntry]
    ///
    /// A [BTreeMap] is used instead of a [std::collections::HashMap], because iteration order is
    /// deterministic, and that's desirable here, since ideally we'd like the program to be able to
    /// parse an object and print it back out, without anything changing.
    pub entries: BTreeMap<String, TreeEntry>,
}

impl Tree {
    pub fn new(entries: BTreeMap<String, TreeEntry>) -> Self {
        Self { entries }
    }

    /// Writes the tree into a writer.
    ///
    /// This simply writes the raw bytes.
    pub fn write<W: Write>(&self, mut _writer: W) -> Result<(), std::io::Error> {
        todo!()
    }
}

/// An entry inside a [Tree].
///
/// This doesn't include the name of the file, because it's the key of the key of the
/// [`Tree::entries`] map.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TreeEntry {
    pub mode: u16,
    pub object: ObjectHash,
}

impl TreeEntry {
    pub fn new(mode: u16, object: ObjectHash) -> Self {
        Self { mode, object }
    }
}
