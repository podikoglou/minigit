use std::{collections::HashSet, io::Write};

use sha1::{
    Digest, Sha1,
    digest::{array::Array, consts::U20},
};

use crate::{
    MinigitError,
    object::hash::ObjectHash,
    storage::{
        Store,
        object::{
            loose::Encode,
            packed::{PackedObject, PackfileHeader, UndeltifiedObject},
        },
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
        let mut writer = HashingWriter::new(writer);

        let header = PackfileHeader::new(
            self.entries
                .len()
                .try_into()
                .map_err(|_| MinigitError::TooMuchData)?,
        );

        header.encode(&mut writer)?;

        for object in self.entries.iter().map(|hash| store.raw_object(hash)) {
            let object = object?;
            let undeltified_object = UndeltifiedObject::new(object);

            // the offset is not actually used in the writing. it's also nontrivial to track the
            // current offset (aka amount of bytes written). perhaps in the future we may neeed to
            // do it though.
            let packed_object = PackedObject::Undeltified(0, undeltified_object);

            packed_object.encode(&mut writer)?;
        }

        let (hash, writer) = writer.finalize();

        writer.write_all(&hash)?;

        Ok(())
    }
}

/// A wrapper of a [Write] that computes a hash of the contents written to it.
struct HashingWriter<W: Write> {
    writer: W,
    hasher: Sha1,
}

impl<W: Write> HashingWriter<W> {
    fn new(writer: W) -> Self {
        Self {
            writer,
            hasher: Sha1::default(),
        }
    }

    fn finalize(self) -> (Array<u8, U20>, W) {
        (self.hasher.finalize(), self.writer)
    }
}

impl<W: Write> Write for HashingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let written = self.writer.write(buf)?;

        self.hasher.update(buf);

        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
