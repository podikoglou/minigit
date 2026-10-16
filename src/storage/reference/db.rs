use std::{fs::File, path::PathBuf, str::FromStr};

use memmap2::Mmap;
use winnow::{
    ModalResult, Parser,
    ascii::{newline, till_line_ending},
    combinator::{opt, seq},
    error::StrContext,
    token::take,
};

use crate::{
    MinigitError,
    error::ParserContext,
    object::hash::{ObjectHash, parse_object_hash_str},
    parsing::Stream,
};

/// Provides a view over a `packed-refs` database file
pub struct RefsDb {
    pub path: PathBuf,
    file: File,
    mmap: Mmap,
}

impl RefsDb {
    /// Opens a `packed-refs` file
    pub fn open(path: PathBuf) -> Result<Self, MinigitError> {
        let file = File::open(&path)?;
        let mmap = unsafe { Mmap::map(&file) }?;

        parse_header
            .parse_next(&mut Stream::new(&mmap))
            .map_err(|err| MinigitError::ParserError(err.to_string(), ParserContext::None))?;

        Ok(Self { path, file, mmap })
    }

    /// Looks up a ref's unpeeled hash
    pub fn lookup(&self, name: &str) -> Result<RefsDbEntry, MinigitError> {
        // This is of couse an inefficient implementation. This does no binary search whatsoever,
        // despite the file being sorted.
        //
        // TODO: implement this using binary search

        let mut stream = Stream::new(&self.mmap);

        while !stream.is_empty() {
            let entry = parse_entry
                .parse_next(&mut stream)
                .map_err(|err| MinigitError::ParserError(err.to_string(), ParserContext::None))?;

            if entry.name == name {
                return Ok(entry);
            }
        }

        Err(MinigitError::RefNotFound)
    }
}

#[derive(Debug)]
pub struct RefsDbEntry {
    /// A hash that points to an object, oftentimes a tag.
    pub hash: ObjectHash,

    pub name: String,

    /// A hash that points directly to the commit object.
    pub peeled_hash: Option<ObjectHash>,
}

impl RefsDbEntry {
    pub fn is_peeled(&self) -> bool {
        self.peeled_hash.is_some()
    }
}

fn parse_header(input: &mut Stream<'_>) -> ModalResult<()> {
    "# pack-refs with: peeled fully-peeled sorted\n"
        .context(StrContext::Label("packed-refs header"))
        .void()
        .parse_next(input)
}

fn parse_entry(input: &mut Stream<'_>) -> ModalResult<RefsDbEntry> {
    seq! {RefsDbEntry {
        hash: parse_object_hash_str.context(StrContext::Label("pointer hash")),
        _: " ",
        name: till_line_ending.map(str::from_utf8).verify_map(Result::ok).map(str::to_owned).context(StrContext::Label("ref name")),
        peeled_hash: opt(seq!(_: "^", parse_object_hash_str, _: newline).map(|x| x.0)).context(StrContext::Label("peeled pointer hash")),
    }}
    .parse_next(input)
}
