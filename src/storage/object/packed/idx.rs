//! This module deals with the packfile index format.

use std::{fs::File, path::PathBuf};

use memmap2::Mmap;
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
};

/// Holds a handle to a memory-mapped pack .idx and provides an API for querying it.
#[derive(Debug)]
pub struct PackIndex {
    pub path: PathBuf,
    pub mmap: Mmap,

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
    pub fn open(path: PathBuf) -> Result<Self, MinigitError> {
        let file = File::open(&path)?;
        let mmap = unsafe { Mmap::map(&file) }?;

        let mut stream = Stream::new(&mmap[..]);

        // read idx header
        //
        // this doesn't return any information, this is mostly for validation that this is a valid idx
        // file.
        header.parse_next(&mut stream).map_err(|err| {
            MinigitError::ParserError(err.to_string(), ParserContext::File(path.clone()))
        })?;

        let fanout_table = fanout_table.parse_next(&mut stream).map_err(|err| {
            MinigitError::ParserError(err.to_string(), ParserContext::File(path.clone()))
        })?;

        let objects_count = fanout_table
            .last()
            .map(|x| Ok(*x as usize))
            .unwrap_or_else(|| Err(MinigitError::InvalidPackIndex))?;

        let object_names = object_names(objects_count)
            .parse_next(&mut stream)
            .map_err(|err| {
                MinigitError::ParserError(err.to_string(), ParserContext::File(path.clone()))
            })?;

        let crc_entries = crc_entries(objects_count)
            .parse_next(&mut stream)
            .map_err(|err| {
                MinigitError::ParserError(err.to_string(), ParserContext::File(path.clone()))
            })?;

        let offsets_1 = offsets_1(objects_count)
            .parse_next(&mut stream)
            .map_err(|err| {
                MinigitError::ParserError(err.to_string(), ParserContext::File(path.clone()))
            })?;

        let offsets_2_entries = offsets_1
            .iter()
            .filter(|entry| (*entry & 0x8000_0000) != 0)
            .count();

        let offsets_2 = offsets_2(offsets_2_entries)
            .parse_next(&mut stream)
            .map_err(|err| {
                MinigitError::ParserError(err.to_string(), ParserContext::File(path.clone()))
            })?;

        let pack_checksum = checksum.parse_next(&mut stream).map_err(|err| {
            MinigitError::ParserError(err.to_string(), ParserContext::File(path.clone()))
        })?;

        let idx_checksum = checksum.parse_next(&mut stream).map_err(|err| {
            MinigitError::ParserError(err.to_string(), ParserContext::File(path.clone()))
        })?;

        Ok(Self {
            path,
            mmap,
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

    /// Gets the object count.
    pub fn objects_count(&self) -> usize {
        self.objects_count
    }

    /// Looks up an object's offset by its hash.
    pub fn lookup(&self, hash: ObjectHash) -> Option<usize> {
        let prefix = Into::<u8>::into(hash.prefix()) as usize;

        let bounds = (prefix.saturating_sub(1), prefix);

        let a = self.fanout_table[bounds.0] as usize;
        let b = self.fanout_table[bounds.1] as usize;

        // amount of objects in the "bucket" we are searching in
        let objects = b - a;

        if objects < 1 {
            return None;
        }

        let search_space = &self.object_names[a..b];

        let offset_idx = search_space.binary_search(&hash).ok()?;

        match self.offsets_1[a + offset_idx] {
            idx if (idx & 0x8000_0000) != 0 => {
                // MSB is set to 1, so this belongs to `self.offsets_2`
                // we mask off the MSB and use `idx` as an index
                Some(self.offsets_2[(idx & 0x7FFF_FFFF) as usize] as usize)
            }
            other => Some(other as usize),
        }
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
