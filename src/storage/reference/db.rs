use std::{fs::File, path::PathBuf};

use memmap2::Mmap;

use crate::MinigitError;

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

        Ok(Self { path, file, mmap })
    }
}
