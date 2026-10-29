use std::{collections::HashMap, fs::File, path::PathBuf};

use memmap2::Mmap;
use winnow::Parser;

use crate::{
    MinigitError,
    error::ParserContext,
    object::hash::ObjectHash,
    parsing::Stream,
    storage::object::{
        ObjectDb, RawObject,
        packed::{header, stream::PackStream},
    },
};

pub struct PackIndexBuilder {
    path: PathBuf,
    mmap: Mmap,

    objects_expected: usize,

    cache: HashMap<ObjectHash, RawObject>,
}

impl PackIndexBuilder {
    pub fn new(path: PathBuf) -> Result<Self, MinigitError> {
        let file = File::open(&path)?;
        let mmap = unsafe { Mmap::map(&file) }?;

        let mut stream = Stream::new(&mmap);

        let header = header.parse_next(&mut stream).map_err(|err| {
            MinigitError::ParserError(err.to_string(), ParserContext::File(path.to_owned()))
        })?;

        Ok(Self {
            path,
            mmap,
            objects_expected: header.objects as usize,
            cache: HashMap::default(),
        })
    }

    pub fn objects(&self) -> PackStream<'_> {
        PackStream::new_from_stream(
            Stream::new(&self.mmap[12..]),
            self.objects_expected,
            ParserContext::File(self.path.clone()),
        )
    }
    pub fn build(&self) {
        todo!()
    }
}

impl ObjectDb for PackIndexBuilder {
    fn raw_object(&self, hash: &ObjectHash) -> Result<RawObject, MinigitError> {
        todo!()
    }
}
