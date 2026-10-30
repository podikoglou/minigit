//! This module deals with packfiles
//!
//! References:
//! - <https://git-scm.com/docs/pack-format>
//! - <https://git-scm.com/book/en/v2/Git-Internals-Packfiles>
//!
//! In particular, this deals with version 2 of the packfile format.

pub mod builder;
pub mod delta;
pub mod idx;
pub mod idx_builder;
mod object;
pub mod stream;

pub use object::*;

use std::{fs::File, path::PathBuf, rc::Rc};

use memmap2::Mmap;
use winnow::{
    ModalResult, Parser,
    binary::be_u32,
    combinator::seq,
    error::StrContext,
    token::{any, take},
};

use crate::{
    TinygitError,
    error::ParserContext,
    object::hash::ObjectHash,
    parsing::{ParseNextExt, ParseResultExt, Stream},
    storage::object::{
        LazyObject, ObjectDb, RawObject,
        loose::Encode,
        packed::{idx::PackIndex, stream::PackStream},
    },
};

/// Holds a handle to an memory-mapped packfile along with its [PackIndex].
///
/// It should be noted that this does not support indexless packfiles. For operations where they are
/// required (which are very rare), one should prefer to use the parser manually.
#[derive(Debug)]
pub struct Packfile {
    pub pack: (PathBuf, Mmap),
    pub idx: PackIndex,

    /// The amount of objects contained in the packfile.
    pub objects_count: u32,

    /// The name of the packfile, excluding the "pack-" extension. This is generally a hash, but we
    /// don't parse it or treat it as such at the moment.
    pub name: String,
}

impl Packfile {
    /// Creates an instance of [Packfile]. This memory maps the pack and (if present) index file,
    /// and parses their headers.
    pub fn open(pack_path: PathBuf, idx_path: PathBuf) -> Result<Self, TinygitError> {
        // open packfile
        let pack_file = File::open(&pack_path)?;
        let pack_buf = unsafe { Mmap::map(&pack_file) }?;

        // extract pack name
        let stem = pack_path.file_stem().unwrap_or_default().to_string_lossy();

        let name = stem
            .strip_prefix("pack-")
            .map(str::to_owned)
            .unwrap_or_else(|| stem.to_string());

        let pack = (pack_path, pack_buf);

        // read packfile header
        let pack_header = header
            .parse(Stream::new(&pack.1[..12]))
            .with_context(ParserContext::File(pack.0.to_owned()))?;

        // if present, open idx
        let idx = PackIndex::open(idx_path)?;

        Ok(Self {
            pack,
            idx,
            name,
            objects_count: pack_header.objects,
        })
    }

    /// Returns an iterator over the objects of the packfile.
    pub fn objects<'a>(&'a self) -> PackStream<'a> {
        let mut buf = Stream::new(&self.pack.1[..]);
        let _: ModalResult<_> = take(12usize).parse_next(&mut buf);

        PackStream::new_from_stream(
            buf,
            self.objects_count as usize,
            ParserContext::File(self.path().to_owned()),
        )
    }

    /// Returns an iterator over the objects of the packfile as [LazyObject]s.
    pub fn lazy_objects(self: Rc<Self>) -> impl Iterator<Item = LazyObject> {
        let count = self.idx.len();

        (0..count).map(move |i| {
            let offset = self.idx.offset_at(i);
            LazyObject::Packed(Rc::clone(&self), offset)
        })
    }

    /// Reads an object at a specific offset of the packfile and returns it.
    ///
    /// This does not resolve the deltas.
    pub fn read_object_at_offset(&self, offset: usize) -> Result<PackedObject, TinygitError> {
        let mut stream = Stream::new(&self.pack.1[..]);
        take(offset)
            .parse_next(&mut stream)
            .context_err(ParserContext::File(self.path().to_owned()), &stream)?;

        object
            .parse_next(&mut stream)
            .context_err(ParserContext::File(self.path().to_owned()), &stream)
    }

    /// Reads an object by its ID ([ObjectHash]) from the packfile and returns it.
    ///
    /// This does not resolve the deltas.
    pub fn read_object_by_id(&self, hash: ObjectHash) -> Result<PackedObject, TinygitError> {
        let offset = self.idx.lookup(hash).ok_or(TinygitError::ObjectNotFound)?;

        self.read_object_at_offset(offset)
    }

    /// Reads and resolves a [`RawObject`] by its ID ([`ObjectHash`]) from the packfile.
    pub fn raw_object(&self, hash: &ObjectHash) -> Result<RawObject, TinygitError> {
        let packed = self.read_object_by_id(hash.clone())?;
        self.resolve(packed)
    }

    /// Resolves a [PackedObject] into a [RawObject], resolving the deltas if it is deltified.
    pub fn resolve(&self, object: PackedObject) -> Result<RawObject, TinygitError> {
        match object {
            PackedObject::Deltified(_offset, object) => {
                let (base, delta) = object.into_inner();

                let base_packed = match base {
                    BaseObject::Ref(id) => self.read_object_by_id(id),
                    BaseObject::Ofs(offset) => self.read_object_at_offset(offset as usize),
                }?;

                let base_raw = self.resolve(base_packed)?;

                Ok(base_raw.apply_delta(delta))
            }

            PackedObject::Undeltified(_, object) => Ok(object.into_inner()),
        }
    }

    /// Returns the path of the packfile.
    pub fn path(&self) -> &PathBuf {
        &self.pack.0
    }
}

impl PartialEq for Packfile {
    fn eq(&self, other: &Self) -> bool {
        self.path() == other.path()
            && self.idx == other.idx
            && self.objects_count == other.objects_count
    }
}

impl Eq for Packfile {}

impl ObjectDb for Packfile {
    fn raw_object(&self, hash: &ObjectHash) -> Result<RawObject, TinygitError> {
        self.raw_object(hash)
    }
}

#[derive(Debug, PartialEq)]
pub struct PackfileHeader {
    /// The amount of objects contained in the packfile.
    pub objects: u32,
}

impl PackfileHeader {
    pub fn new(objects: u32) -> Self {
        Self { objects }
    }
}

impl Encode for PackfileHeader {
    fn encode<W: std::io::prelude::Write>(&self, writer: &mut W) -> Result<(), TinygitError> {
        write!(writer, "PACK")?;
        writer.write_all(&[0x00, 0x00, 0x00, 0x02])?;
        writer.write_all(&self.objects.to_be_bytes())?;

        Ok(())
    }
}

/// Parses the header of a packfile, returning the amount of objects contained in the packfile.
pub fn header(input: &mut Stream<'_>) -> ModalResult<PackfileHeader> {
    seq! {PackfileHeader {
        _: "PACK".context(StrContext::Label("magic bytes")),
        _: seq!(0x00, 0x00, 0x00, 0x02).context(StrContext::Label("packfile version")),
        objects: be_u32.context(StrContext::Label("objects amount"))
    }}
    .context(StrContext::Label("packfile header"))
    .parse_next(input)
}

pub fn offset(input: &mut Stream<'_>) -> ModalResult<u64> {
    let mut current_byte = any.parse_next(input)?;
    let mut value = (current_byte & 0b0111_1111) as u64;

    while current_byte >= 128 {
        current_byte = any.parse_next(input)?;

        let chunk = (current_byte & 0b0111_1111) as u64;

        match value.checked_add(1) {
            Some(val) => value = val,
            None => break,
        }
        value = (value << 7) | chunk;
    }

    Ok(value)
}

pub fn size(input: &mut Stream<'_>) -> ModalResult<u64> {
    let mut current_byte = any.parse_next(input)?;
    let mut value = (current_byte & 0b0111_1111) as u64;
    let mut pos = 7;

    while current_byte >= 128 && pos < 70 {
        current_byte = any.parse_next(input)?;

        // these are the bits that constitute the size, which is the byte as-is, but with the first
        // bit masked off as it's the continuation bit
        let chunk = (current_byte & 0b0111_1111) as u64;

        value |= chunk << pos;
        pos += 7;
    }

    Ok(value)
}
