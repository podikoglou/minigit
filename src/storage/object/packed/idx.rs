//! This module deals with the packfile index format.

use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use sha1::{
    Digest, Sha1,
    digest::{array::Array, consts::U20},
};
use winnow::{
    ModalResult, Parser,
    binary::{be_u32, be_u64},
    combinator::{repeat, seq},
    error::StrContext,
};

use crate::{
    MinigitError,
    error::ParserContext,
    object::hash::{ObjectHash, parse_object_hash},
    parsing::Stream,
    storage::object::loose::Encode,
};

/// A packfile index providing information for reading a Packfile.
#[derive(Debug, Default)]
pub struct PackIndex {
    pub path: PathBuf,

    pub objects_count: usize,
    pub fanout_table: Vec<u32>,
    pub object_names: Vec<ObjectHash>,
    pub crc_entries: Vec<u32>,
    pub offsets_1: Vec<u32>,
    pub offsets_2: Vec<u64>,
    pub pack_checksum: ObjectHash,
    pub idx_checksum: ObjectHash,
}

impl PartialEq for PackIndex {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl PackIndex {
    /// Creates an index of [PackIndex], memory mapping the file and reading its header.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, MinigitError> {
        let contents = fs::read(path.as_ref())?;
        let mut stream = Stream::new(&contents[..]);

        // read idx header
        //
        // this doesn't return any information, this is mostly for validation that this is a valid idx
        // file.
        header.parse_next(&mut stream).map_err(|err| {
            MinigitError::ParserError(
                err.to_string(),
                ParserContext::File(path.as_ref().to_path_buf()),
            )
        })?;

        let fanout_table = fanout_table.parse_next(&mut stream).map_err(|err| {
            MinigitError::ParserError(
                err.to_string(),
                ParserContext::File(path.as_ref().to_path_buf()),
            )
        })?;

        let objects_count = fanout_table
            .last()
            .map(|x| Ok(*x as usize))
            .unwrap_or_else(|| Err(MinigitError::InvalidPackIndex))?;

        let object_names = object_names(objects_count)
            .parse_next(&mut stream)
            .map_err(|err| {
                MinigitError::ParserError(
                    err.to_string(),
                    ParserContext::File(path.as_ref().to_path_buf()),
                )
            })?;

        let crc_entries = crc_entries(objects_count)
            .parse_next(&mut stream)
            .map_err(|err| {
                MinigitError::ParserError(
                    err.to_string(),
                    ParserContext::File(path.as_ref().to_path_buf()),
                )
            })?;

        let offsets_1 = offsets_1(objects_count)
            .parse_next(&mut stream)
            .map_err(|err| {
                MinigitError::ParserError(
                    err.to_string(),
                    ParserContext::File(path.as_ref().to_path_buf()),
                )
            })?;

        let offsets_2_entries = offsets_1
            .iter()
            .filter(|entry| (*entry & 0x8000_0000) != 0)
            .count();

        let offsets_2 = offsets_2(offsets_2_entries)
            .parse_next(&mut stream)
            .map_err(|err| {
                MinigitError::ParserError(
                    err.to_string(),
                    ParserContext::File(path.as_ref().to_path_buf()),
                )
            })?;

        let pack_checksum = checksum.parse_next(&mut stream).map_err(|err| {
            MinigitError::ParserError(
                err.to_string(),
                ParserContext::File(path.as_ref().to_path_buf()),
            )
        })?;

        let idx_checksum = checksum.parse_next(&mut stream).map_err(|err| {
            MinigitError::ParserError(
                err.to_string(),
                ParserContext::File(path.as_ref().to_path_buf()),
            )
        })?;

        Ok(Self {
            path: path.as_ref().to_path_buf(),
            objects_count,
            fanout_table,
            object_names,
            crc_entries,
            offsets_1,
            offsets_2,
            pack_checksum,
            idx_checksum,
        })
    }

    /// Creates a new [PackIndex] meant for creating a new index at the given path.
    ///
    /// It does not read from path, it is only stored for writing.
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            ..Default::default()
        }
    }

    /// Writes (overwriting contents) the index to the filesystem.
    pub fn write(&self) -> Result<(), MinigitError> {
        let mut file = File::create(&self.path)?;

        self.encode(&mut file)?;

        Ok(())
    }

    /// Gets the object count.
    pub fn objects_count(&self) -> usize {
        self.objects_count
    }

    /// Looks up an object's offset by its hash.
    pub fn lookup(&self, hash: ObjectHash) -> Option<usize> {
        let prefix = Into::<u8>::into(hash.prefix()) as usize;

        let a = if prefix == 0 {
            0
        } else {
            self.fanout_table[prefix - 1] as usize
        };
        let b = self.fanout_table[prefix] as usize;

        // amount of objects in the "bucket" we are searching in
        let objects = b - a;

        if objects < 1 {
            return None;
        }

        let search_space = &self.object_names[a..b];

        let offset_idx = search_space.binary_search(&hash).ok()?;

        Some(self.offset_at(a + offset_idx))
    }

    /// Returns the offset for the object at the given index in the packfile.
    pub fn offset_at(&self, idx: usize) -> usize {
        let offset = self.offsets_1[idx];
        if (offset & 0x8000_0000) != 0 {
            self.offsets_2[(offset & 0x7FFF_FFFF) as usize] as usize
        } else {
            offset as usize
        }
    }

    /// Returns the number of indexed objects.
    pub fn len(&self) -> usize {
        self.offsets_1.len()
    }

    pub fn is_empty(&self) -> bool {
        self.offsets_1.is_empty()
    }
}

impl Encode for PackIndex {
    fn encode<W: std::io::prelude::Write>(&self, writer: &mut W) -> Result<(), MinigitError> {
        let mut writer = HashingWriter::new(writer);

        writer.write_all(&[0xff, 0x74, 0x4f, 0x63])?; // magic bytes
        writer.write_all(&[0x00, 0x00, 0x00, 0x02])?; // version

        // fanout table
        for entry in &self.fanout_table {
            writer.write_all(&entry.to_be_bytes())?;
        }

        // object names
        for hash in &self.object_names {
            writer.write_all(hash.into())?;
        }

        // CRC values
        for crc in &self.crc_entries {
            writer.write_all(&crc.to_be_bytes())?;
        }

        // small offsets table
        for entry in &self.offsets_1 {
            writer.write_all(&entry.to_be_bytes())?;
        }

        // large offsets table
        for entry in &self.offsets_2 {
            writer.write_all(&entry.to_be_bytes())?;
        }

        // packfile hash
        writer.write_all((&self.pack_checksum).into())?;

        // hash of the entire index
        let (hash, writer) = writer.finalize();

        writer.write_all(&hash)?;

        Ok(())
    }
}

pub fn header(input: &mut Stream<'_>) -> ModalResult<()> {
    seq!(
        seq!(0xff, 0x74, 0x4f, 0x63).context(StrContext::Label("magic bytes")),
        seq!(0x00, 0x00, 0x00, 0x02).context(StrContext::Label("version"))
    )
    .void()
    .parse_next(input)
}

pub fn fanout_table(input: &mut Stream<'_>) -> ModalResult<Vec<u32>> {
    // TODO: should we do some basic validation here to ensure they are cumulative? (because if
    // they're not, we're probably reading something wrong or the packfile is totally garbage)
    repeat(256, be_u32)
        .context(StrContext::Label("fanout table"))
        .parse_next(input)
}

pub fn object_names(amount: usize) -> impl FnMut(&mut Stream<'_>) -> ModalResult<Vec<ObjectHash>> {
    move |input: &mut Stream<'_>| repeat(amount, parse_object_hash).parse_next(input)
}

pub fn crc_entries(amount: usize) -> impl FnMut(&mut Stream<'_>) -> ModalResult<Vec<u32>> {
    move |input: &mut Stream<'_>| repeat(amount, be_u32).parse_next(input)
}

// significant part about this: each entry here should use up to 31 bytes.
// if it uses more than 31 bytes, i.e. the MSB is 1, then this is not an offset,
// but an index (with the MSB 0) to the
pub fn offsets_1(amount: usize) -> impl FnMut(&mut Stream<'_>) -> ModalResult<Vec<u32>> {
    move |input: &mut Stream<'_>| repeat(amount, be_u32).parse_next(input)
}

// this is the second offsets table. the amount of entries this has depends on the first table. in
// particular it depends on the amount of entries it has that have their MSB set to 1.
pub fn offsets_2(amount: usize) -> impl FnMut(&mut Stream<'_>) -> ModalResult<Vec<u64>> {
    move |input: &mut Stream<'_>| repeat(amount, be_u64).parse_next(input)
}

pub fn checksum(input: &mut Stream<'_>) -> ModalResult<ObjectHash> {
    parse_object_hash(input)
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
