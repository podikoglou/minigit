use std::{collections::BTreeMap, io::Write};

use crate::storage::object::LazyObject;

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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TreeEntry {
    pub mode: u16,
    pub object: LazyObject,
}
