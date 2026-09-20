use std::{collections::BTreeMap, io::Write};

use sha1::digest::{array::Array, consts::U20};
use winnow::{
    ModalResult, Parser,
    combinator::{repeat, seq},
    error::StrContext,
};

use crate::{
    fs::{FileName, parse_file_name},
    object::hash::{ObjectHash, parse_object_hash},
    parsing::{Stream, mode},
    storage::object::loose::WriteLoose,
};

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
    #[must_use]
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

/// Parses a tree object from some bytes.
pub fn parse_tree(input: &mut Stream<'_>) -> ModalResult<Tree> {
    // NOTE: not sure if this should be `0..` or `1..`
    // should we be able to parse empty trees?
    repeat(0.., parse_tree_entry)
        .map(|entries: Vec<(u16, FileName, ObjectHash)>| {
            entries
                .into_iter()
                .map(|(mode, name, hash)| (name, TreeEntry::new(mode, hash)))
                .collect::<BTreeMap<FileName, TreeEntry>>()
        })
        .context(StrContext::Label("tree object"))
        .map(Tree::new)
        .parse_next(input)
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
    #[must_use]
    pub fn new(mode: u16, object: ObjectHash) -> Self {
        Self { mode, object }
    }
}

impl WriteLoose for (&FileName, &TreeEntry) {
    fn write_loose<W: Write>(&self, writer: &mut W) -> Result<(), crate::MinigitError> {
        write!(writer, "{:o} {}\0", self.1.mode, self.0)?;

        let hash_s = Into::<Array<u8, U20>>::into(self.1.object.clone());
        writer.write_all(hash_s.as_slice())?;

        Ok(())
    }
}
/// Parses a tree object's entry into a tuple `(mode, name, hash)` from some bytes.
pub fn parse_tree_entry(input: &mut Stream<'_>) -> ModalResult<(u16, FileName, ObjectHash)> {
    seq!((mode, _: " ", parse_file_name, parse_object_hash))
        .context(StrContext::Label("tree entry"))
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use winnow::Parser;

    use crate::object::tree::parse_tree_entry;

    #[test]
    fn tree_entry_parses_valid_entries() {
        assert_eq!(
            parse_tree_entry.parse_peek(b"100644 cli.rs\0\x29\xf3\x23\xb3\x1a\xd1\x29\x96\x4f\xfb\x4f\x97\xf2\x03\xbe\x9c\x2f\x35\x10\x7d"),
            Ok((&b""[..], (0o100644, "cli.rs\0".parse().unwrap(), [0x29, 0xf3, 0x23, 0xb3, 0x1a, 0xd1, 0x29, 0x96, 0x4f, 0xfb, 0x4f, 0x97, 0xf2, 0x03, 0xbe, 0x9c, 0x2f, 0x35, 0x10, 0x7d].into() )))
        );
    }
}
