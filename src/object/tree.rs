use std::{collections::BTreeMap, io::Write};

use sha1::digest::{array::Array, consts::U20};

use crate::{fs::FileName, object::hash::ObjectHash, storage::object::loose::WriteLoose};

/// A tree: an object that associates file names to [tree entries](TreeEntry).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Tree {
    /// A mapping from [String] -> [TreeEntry]
    ///
    /// A [BTreeMap] is used instead of a [std::collections::HashMap], because iteration order is
    /// deterministic, and that's desirable here, since ideally we'd like the program to be able to
    /// parse an object and print it back out, without anything changing.
    pub entries: BTreeMap<FileName, TreeEntry>,
}

impl Tree {
    pub fn new(entries: BTreeMap<FileName, TreeEntry>) -> Self {
        Self { entries }
    }
}

impl WriteLoose for Tree {
    fn write_loose<W: Write>(&self, writer: &mut W) -> Result<(), crate::MinigitError> {
        for entry in &self.entries {
            entry.write_loose(writer)?;
        }

        Ok(())
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

impl WriteLoose for (&FileName, &TreeEntry) {
    fn write_loose<W: Write>(&self, writer: &mut W) -> Result<(), crate::MinigitError> {
        write!(writer, "{:} {}\0", self.1.mode, self.0)?;

        let hash_s = Into::<Array<u8, U20>>::into(self.1.object.clone());
        writer.write_all(hash_s.as_slice())?;

        Ok(())
    }
}
