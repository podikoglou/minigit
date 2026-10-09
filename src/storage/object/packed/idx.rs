//! This module deals with the packfile index format.

use std::{
    fs::File,
    path::PathBuf,
    sync::{LazyLock, OnceLock},
};

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

    fanout_table: OnceLock<Result<Vec<u32>, MinigitError>>,
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

        // read idx header
        //
        // this return any information, this is mostly for validation that this is a valid idx
        // file.
        header.parse(Stream::new(&mmap[..12])).map_err(|err| {
            MinigitError::ParserError(err.to_string(), ParserContext::File(path.clone()))
        })?;

        Ok(Self {
            path,
            mmap,
            fanout_table: Default::default(),
        })
    }

    /// Creates a new [Stream] starting at a given offset. Convenient helper used by other functions
    /// here.
    #[inline(always)]
    fn stream_from<'a>(&'a self, offset: usize) -> Stream<'a> {
        Stream::new(&self.mmap[offset..])
    }

    /// Lazily reads the fanout table.
    pub fn fanout_table(&self) -> Result<&Vec<u32>, &MinigitError> {
        self.fanout_table
            .get_or_init(|| {
                fanout_table.parse(self.stream_from(12)).map_err(|err| {
                    MinigitError::ParserError(
                        err.to_string(),
                        ParserContext::File(self.path.clone()),
                    )
                })
            })
            .as_ref()
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
