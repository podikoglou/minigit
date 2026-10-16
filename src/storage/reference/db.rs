use std::{fs::File, path::PathBuf};

use memmap2::Mmap;
use winnow::{ModalResult, Parser};

use crate::{MinigitError, error::ParserContext, object::hash::ObjectHash, parsing::Stream};

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
    pub fn lookup(&self, name: &str) -> Option<ObjectHash> {
        todo!()
    }
}

fn parse_header(input: &mut Stream<'_>) -> ModalResult<()> {
    "# pack-refs with: peeled fully-peeled sorted\n"
        .void()
        .parse_next(input)
}
