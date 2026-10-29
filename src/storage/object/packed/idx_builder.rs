use std::{collections::HashMap, fs::File, path::PathBuf};

use fallible_iterator::FallibleIterator;
use memmap2::Mmap;
use winnow::{
    Parser,
    error::{ContextError, ErrMode},
    token::take,
};

use crate::{
    MinigitError,
    error::ParserContext,
    object::hash::ObjectHash,
    parsing::Stream,
    storage::object::{
        ObjectDb, RawObject,
        packed::{BaseObject, PackedObject, header, object, stream::PackStream},
    },
};

pub struct PackIndexBuilder {
    path: PathBuf,
    mmap: Mmap,

    objects_expected: usize,
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
        })
    }

    fn objects(&self) -> PackStream<'_> {
        PackStream::new_from_stream(
            Stream::new(&self.mmap[12..]),
            self.objects_expected,
            ParserContext::File(self.path.clone()),
        )
    }

    fn read_object_at_offset(&self, offset: usize) -> Result<PackedObject, MinigitError> {
        let mut stream = Stream::new(&self.mmap[..]);

        take(offset)
            .parse_next(&mut stream)
            .map_err(|err: ErrMode<ContextError>| {
                MinigitError::ParserError(
                    err.to_string(),
                    ParserContext::File(self.path.to_owned()),
                )
            })?;

        object.parse_next(&mut stream).map_err(|err| {
            MinigitError::ParserError(err.to_string(), ParserContext::File(self.path.to_owned()))
        })
    }

    /// Recursively resolves a [PackedObject] to a [RawObject], possibly using the cache (thus the
    /// need for self to be mutable)
    fn resolve(
        &self,
        object: PackedObject,
        cache: &mut ObjectCache,
    ) -> Result<RawObject, MinigitError> {
        let offset = object.offset();

        let raw_object = match object {
            PackedObject::Deltified(_offset, object) => {
                let (base, delta) = object.into_inner();

                let base_raw = match base {
                    BaseObject::Ref(id) => self.raw_object(&id),

                    BaseObject::Ofs(offset) => {
                        let offset = offset as usize;

                        Ok(match cache.get(&offset) {
                            // already cached
                            Some(val) => val.clone(),

                            // read at offset, cache for future use
                            None => {
                                let packed = self.read_object_at_offset(offset)?;
                                let raw = self.resolve(packed, cache)?;

                                raw
                            }
                        })
                    }
                };

                base_raw.map(|base| base.apply_delta(delta))
            }

            PackedObject::Undeltified(_, object) => Ok(object.into_inner()),
        }?;

        cache.insert(offset, raw_object.clone());

        Ok(raw_object)
    }

    pub fn build(&mut self, cache: &mut ObjectCache) -> Result<(), MinigitError> {
        for packed_object in self.objects().iterator() {
            let packed_object = packed_object?;
            let raw_object = self.resolve(packed_object, cache)?;

            todo!()
        }

        Ok(())
    }
}

impl ObjectDb for PackIndexBuilder {
    fn raw_object(&self, hash: &ObjectHash) -> Result<RawObject, MinigitError> {
        // we could have an in memory cache that maps hashes to raw objects like we do
        // with offsets, but i believe ref deltas are mostly used when the object is
        // outside the pack, which is a feature we do not yet support and would be hard
        // to implement
        todo!("raw_object for pack index builder")
    }
}

/// Caches objects by their offsets (TODO: also hash?)
pub struct ObjectCache {
    offset_to_obj: HashMap<usize, RawObject>,
}

impl ObjectCache {
    pub fn get(&mut self, offset: &usize) -> Option<RawObject> {
        self.offset_to_obj.get(offset).map(|x| x.to_owned())
    }

    fn insert(&mut self, offset: usize, object: RawObject) {
        self.offset_to_obj.insert(offset, object);
    }
}
