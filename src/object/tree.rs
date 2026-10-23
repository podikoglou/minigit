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
    storage::object::loose::Encode,
};

/// The key for a [`Tree`] entry.
///
/// # Sorting
///
/// Git compares tree entry names by appending a virtual `'/'` to any entry
/// whose mode is a directory (`040000`). This means a directory called `foo`
/// sorts *after* `foo-bar` (because `'/' > '-'`), unlike plain lexicographic
/// order where `foo` would come first.
///
/// The algorithm: compare byte-by-byte up to the shorter name, then treat the
/// "next byte" as `'/'` for directories and `'\0'` for everything else.
///
/// This is implemented in the [`Ord`] implementation.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TreeKey {
    pub name: FileName,
    pub mode: u16,
}

impl TreeKey {
    #[must_use]
    pub fn new(name: FileName, mode: u16) -> Self {
        Self { name, mode }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.name.as_str()
    }

    /// The virtual trailing byte git uses when comparing this entry.
    /// Directories sort as `name + '/'`; everything else as `name + '\0'`.
    fn tail(&self) -> u8 {
        if self.mode == 0o40000 { b'/' } else { b'\0' }
    }
}

impl PartialOrd for TreeKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TreeKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let a = self.name.as_str().as_bytes();
        let b = other.name.as_str().as_bytes();
        let min_len = a.len().min(b.len());

        // Compare the common prefix first.
        match a[..min_len].cmp(&b[..min_len]) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        // After the common prefix, pit the "next byte" of each side against
        // the other: the actual next character if the name is longer, or the
        // virtual tail byte ('/' or '\0') if we've reached the end.
        let a_next = a.get(min_len).copied().unwrap_or_else(|| self.tail());
        let b_next = b.get(min_len).copied().unwrap_or_else(|| other.tail());
        a_next.cmp(&b_next)
    }
}

/// A tree: an object that associates file names to [tree entries](TreeEntry).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Tree {
    /// A mapping from [`TreeKey`] -> [`TreeEntry`].
    ///
    /// A [`BTreeMap`] is used instead of a [`std::collections::HashMap`], because iteration order is
    /// deterministic, and that's desirable here, since ideally we'd like the program to be able to
    /// parse an object and print it back out, without anything changing.
    pub entries: BTreeMap<TreeKey, TreeEntry>,
}

impl Tree {
    #[must_use]
    pub fn new(entries: BTreeMap<TreeKey, TreeEntry>) -> Self {
        Self { entries }
    }
}

impl Encode for Tree {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), crate::MinigitError> {
        for entry in &self.entries {
            entry.encode(writer)?;
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
                .map(|(mode, name, hash)| (TreeKey::new(name, mode), TreeEntry::new(hash)))
                .collect::<BTreeMap<TreeKey, TreeEntry>>()
        })
        .context(StrContext::Label("tree object"))
        .map(Tree::new)
        .parse_next(input)
}

/// An entry inside a [Tree].
///
/// The name and mode live in [`TreeKey`]; this holds only the object hash.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TreeEntry {
    pub object: ObjectHash,
}

impl TreeEntry {
    #[must_use]
    pub fn new(object: ObjectHash) -> Self {
        Self { object }
    }
}

impl Encode for (&TreeKey, &TreeEntry) {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), crate::MinigitError> {
        write!(writer, "{:o} {}\0", self.0.mode, self.0.name)?;

        let hash_s = Into::<Array<u8, U20>>::into(self.1.object.clone());
        writer.write_all(hash_s.as_slice())?;

        Ok(())
    }
}

/// Parses a tree object's entry into a tuple `(mode, name, hash)` from some bytes.
pub fn parse_tree_entry(input: &mut Stream<'_>) -> ModalResult<(u16, FileName, ObjectHash)> {
    seq!(mode, _: " ", parse_file_name, parse_object_hash)
        .context(StrContext::Label("tree entry"))
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use winnow::Parser;

    use crate::{object::tree::parse_tree_entry, parsing::Stream};

    #[test]
    fn tree_entry_parses_valid_entries() {
        assert_eq!(
            parse_tree_entry
                .parse_peek(Stream::new(b"100644 cli.rs\0\x29\xf3\x23\xb3\x1a\xd1\x29\x96\x4f\xfb\x4f\x97\xf2\x03\xbe\x9c\x2f\x35\x10\x7d"))
                .map(|(r, v)| (*r, v)),
            Ok((&b""[..], (0o100644, "cli.rs\0".parse().unwrap(), [0x29, 0xf3, 0x23, 0xb3, 0x1a, 0xd1, 0x29, 0x96, 0x4f, 0xfb, 0x4f, 0x97, 0xf2, 0x03, 0xbe, 0x9c, 0x2f, 0x35, 0x10, 0x7d].into() )))
        );
    }

    #[test]
    fn tree_key_git_sort_order() {
        use crate::object::tree::TreeKey;

        // A directory "foo" should sort AFTER "foo-bar" because git compares
        // "foo/" vs "foo-", and '/' (0x2F) > '-' (0x2D).
        let dir_foo: TreeKey = TreeKey::new("foo\0".parse().unwrap(), 0o40000);
        let file_foo_bar: TreeKey = TreeKey::new("foo-bar\0".parse().unwrap(), 0o100644);
        assert!(
            dir_foo > file_foo_bar,
            "directory 'foo' should sort after 'foo-bar'"
        );

        // A plain file "foo" should sort BEFORE "foo-bar" (normal lex order).
        let file_foo: TreeKey = TreeKey::new("foo\0".parse().unwrap(), 0o100644);
        assert!(
            file_foo < file_foo_bar,
            "file 'foo' should sort before 'foo-bar'"
        );
    }
}
