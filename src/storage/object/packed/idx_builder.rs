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
        packed::{BaseObject, PackedObject, header, idx::PackIndex, object, stream::PackStream},
    },
};

#[derive(Debug)]
pub struct PackIndexBuilder {
    pack_path: PathBuf,
    index_path: PathBuf,
    mmap: Mmap,

    objects_expected: usize,
}

impl PackIndexBuilder {
    /// Creates a new pack builder, memory-mapping the packfile and parsing its header.
    ///
    /// If `index_path` is not provided, it is inferred from the pack's path, where the `.pack`
    /// extension is replaced with `.idx`
    pub fn new(pack_path: PathBuf, index_path: Option<PathBuf>) -> Result<Self, MinigitError> {
        let index_path = index_path.unwrap_or_else(|| pack_path.with_extension("idx"));

        let file = File::open(&pack_path)?;
        let mmap = unsafe { Mmap::map(&file) }?;

        let mut stream = Stream::new(&mmap);

        let header = header.parse_next(&mut stream).map_err(|err| {
            MinigitError::ParserError(err.to_string(), ParserContext::File(pack_path.to_owned()))
        })?;

        Ok(Self {
            pack_path,
            index_path,
            mmap,
            objects_expected: header.objects as usize,
        })
    }

    fn objects(&self) -> PackStream<'_> {
        PackStream::new_from_stream(
            Stream::new(&self.mmap[12..]),
            self.objects_expected,
            ParserContext::File(self.pack_path.clone()),
        )
    }

    fn read_object_at_offset(&self, offset: usize) -> Result<PackedObject, MinigitError> {
        let mut stream = Stream::new(&self.mmap[..]);

        take(offset)
            .parse_next(&mut stream)
            .map_err(|err: ErrMode<ContextError>| {
                MinigitError::ParserError(
                    err.to_string(),
                    ParserContext::File(self.pack_path.to_owned()),
                )
            })?;

        object.parse_next(&mut stream).map_err(|err| {
            MinigitError::ParserError(
                err.to_string(),
                ParserContext::File(self.pack_path.to_owned()),
            )
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
                            None => self
                                .read_object_at_offset(offset)
                                .and_then(|packed| self.resolve(packed, cache))?,
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

    pub fn build(&self, cache: &mut ObjectCache) -> Result<PackIndex, MinigitError> {
        let mut idx = PackIndex::new(&self.index_path);

        let mut intermediate: Vec<IntermediateTableEntry> = Vec::default();

        for packed_object in self.objects().iterator() {
            let packed_object = packed_object?;

            // wait, why is this usize? what about 32 bit computers? can we address large
            // packfiles in them at all?
            let offset = packed_object.offset();

            let raw_object = self.resolve(packed_object, cache)?;

            let hash = raw_object.hash()?;

            let prefix: u8 = hash.prefix().into();
            let prefix = prefix as usize;

            // populate fanout table cumulatively
            for i in prefix..idx.fanout_table.len() {
                if let Some(entry) = idx.fanout_table.get_mut(i) {
                    *entry += 1;
                }
            }

            idx.objects_count = idx.fanout_table[255] as usize;

            // populate object names
            idx.object_names.push(hash.clone());

            let crc = crc32fast::hash(&raw_object.bytes);

            // figure out where to place offset
            let offset = if offset > 2 ^ 31 {
                let offsets_2_idx = idx.offsets_2.len();

                idx.offsets_2.push(offset as u64);

                Offset::Large(offsets_2_idx as u32)
            } else {
                Offset::Small(offset as u32)
            };

            intermediate.push(IntermediateTableEntry(hash, crc, offset));
        }

        // does this sort as we expect it to?
        intermediate.sort();

        for IntermediateTableEntry(_, crc, offset) in intermediate {
            idx.crc_entries.push(crc);

            match offset {
                Offset::Small(offset) => idx.offsets_1.push(offset),
                Offset::Large(index) => idx.offsets_1.push(index | 0x10),
            }
        }

        Ok(idx)
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
#[derive(Default, Debug)]
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

/// A value of the first offset table.
///
/// Either an offset to the Packfile ([Offset::Small]) or an index to the second table
/// ([Offset::Large]).
#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
enum Offset {
    /// The offset of the object in the Packfile.
    Small(u32),

    /// The index to the large offsets table. It is assumed that it exists.
    Large(u32),
}

impl From<Offset> for u32 {
    fn from(val: Offset) -> Self {
        match val {
            Offset::Small(offset) => offset,
            Offset::Large(index) => index | 0x10,
        }
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq)]
struct IntermediateTableEntry(ObjectHash, u32, Offset);
