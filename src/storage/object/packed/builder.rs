use std::{collections::HashSet, io::Write};

use crate::{
    MinigitError,
    object::hash::ObjectHash,
    storage::{
        Store,
        object::{loose::Encode, packed::PackfileHeader},
    },
};

/// A data structure for building and writing packfiles to a [Write].
#[derive(Default)]
pub struct PackBuilder {
    /// A collection containing the IDs of the objects to be included in the packfile.
    ///
    /// No guarantee is made about whether these objects exist. They are only validated upon
    /// flushing.
    entries: HashSet<ObjectHash>,
}

impl PackBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an entry to be included in the packfile.
    pub fn insert(&mut self, hash: ObjectHash) {
        self.entries.insert(hash);
    }

    /// Writes the packfile into the writer.
    pub fn flush<W: Write>(&self, writer: &mut W, store: &Store) -> Result<(), MinigitError> {
        let header = PackfileHeader::new(
            self.entries
                .len()
                .try_into()
                .map_err(|_| MinigitError::TooMuchData)?,
        );

        header.encode(writer)?;

        Ok(())
    }
}
