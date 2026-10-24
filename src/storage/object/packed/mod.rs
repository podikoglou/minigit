//! This module deals with packfiles
//!
//! References:
//! - <https://git-scm.com/docs/pack-format>
//! - <https://git-scm.com/book/en/v2/Git-Internals-Packfiles>
//!
//! In particular, this deals with version 2 of the packfile format.

pub mod delta;
pub mod idx;
mod object;

pub use object::*;

use std::{fs::File, path::PathBuf, rc::Rc};

use fallible_iterator::FallibleIterator;
use memmap2::Mmap;
use winnow::{
    ModalResult, Parser,
    binary::be_u32,
    combinator::seq,
    error::{ContextError, ErrMode, StrContext},
    token::{any, take},
};

use crate::{
    MinigitError,
    error::ParserContext,
    object::hash::ObjectHash,
    parsing::Stream,
    storage::object::{
        LazyObject, RawObject,
        packed::{
            delta::{CopyInstruction, InsertInstruction, Instruction},
            idx::PackIndex,
        },
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
    pub fn open(pack_path: PathBuf, idx_path: PathBuf) -> Result<Self, MinigitError> {
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
        let pack_header = header.parse(Stream::new(&pack.1[..12])).map_err(|err| {
            MinigitError::ParserError(err.to_string(), ParserContext::File(pack.0.to_owned()))
        })?;

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
    pub fn objects<'a>(&'a self) -> Objects<'a> {
        let mut buf = Stream::new(&self.pack.1[..]);
        let _: ModalResult<_> = take(12usize).parse_next(&mut buf);

        Objects {
            amount: self.objects_count as usize,
            context: ParserContext::File(self.path().to_owned()),
            read: 0,
            buf,
        }
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
    pub fn read_object_at_offset(&self, offset: usize) -> Result<PackedObject, MinigitError> {
        let mut stream = Stream::new(&self.pack.1[..]);
        take(offset)
            .parse_next(&mut stream)
            .map_err(|err: ErrMode<ContextError>| {
                MinigitError::ParserError(
                    err.to_string(),
                    ParserContext::File(self.path().to_owned()),
                )
            })?;

        object.parse_next(&mut stream).map_err(|err| {
            MinigitError::ParserError(err.to_string(), ParserContext::File(self.path().to_owned()))
        })
    }

    /// Reads an object by its ID ([ObjectHash]) from the packfile and returns it.
    ///
    /// This does not resolve the deltas.
    pub fn read_object_by_id(&self, hash: ObjectHash) -> Result<PackedObject, MinigitError> {
        let offset = self.idx.lookup(hash).ok_or(MinigitError::ObjectNotFound)?;

        self.read_object_at_offset(offset)
    }

    /// Resolves a [PackedObject] into a [RawObject], resolving the deltas if it is deltified.
    pub fn resolve(&self, object: PackedObject) -> Result<RawObject, MinigitError> {
        match object {
            PackedObject::Deltified(_offset, object) => {
                let (base, delta) = object.into_inner();

                let base_packed = match base {
                    BaseObject::Ref(id) => self.read_object_by_id(id),
                    BaseObject::Ofs(offset) => self.read_object_at_offset(offset as usize),
                }?;

                let base_raw = self.resolve(base_packed)?;

                let mut new_bytes: Vec<u8> =
                    Vec::with_capacity(delta.deltified_object_size as usize);

                for instruction in delta.instructions {
                    let mut new_data = match instruction {
                        Instruction::Insert(InsertInstruction(data)) => data,
                        Instruction::Copy(CopyInstruction { offset, size }) => {
                            base_raw.bytes[offset as usize..offset as usize + size as usize].into()
                        }
                    };

                    new_bytes.append(&mut new_data);
                }

                Ok(RawObject::new(base_raw.r#type, new_bytes))
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

/// An iterator over [PackedObject].
pub struct Objects<'a> {
    /// The amount of objects to read. [`Self::next`] will start returning `None` after having read
    /// `amount` objects.
    amount: usize,

    /// The parser context, used for error reporting.
    context: ParserContext,

    /// A slice of the packfile's contents, starting from the first object.
    buf: Stream<'a>,

    /// (state) The amount of objects that have been read so far.
    read: usize,
}

impl FallibleIterator for Objects<'_> {
    type Item = PackedObject;
    type Error = MinigitError;

    fn next(&mut self) -> Result<Option<PackedObject>, MinigitError> {
        if self.read >= self.amount {
            return Ok(None);
        }

        let object = object
            .parse_next(&mut self.buf)
            .map_err(|err| MinigitError::ParserError(err.to_string(), self.context.to_owned()))?;

        self.read += 1;

        Ok(Some(object))
    }
}

impl Eq for Packfile {}

#[derive(Debug)]
pub struct PackfileHeader {
    /// The amount of objects contained in the packfile.
    pub objects: u32,
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

fn offset(input: &mut Stream<'_>) -> ModalResult<u64> {
    let mut current_byte = any.parse_next(input)?;
    let mut offset = (current_byte & 0b0111_1111) as u64;

    while current_byte >= 128 {
        current_byte = any.parse_next(input)?;

        let chunk = (current_byte & 0b0111_1111) as u64;

        offset += 1;
        offset = (offset << 7) | chunk;
    }

    Ok(offset)
}

pub fn size(input: &mut Stream<'_>) -> ModalResult<u64> {
    // TODO: check to ensure this fits inside a u64
    let mut current_byte = any.parse_next(input)?;
    let mut value = (current_byte & 0b0111_1111) as u64;
    let mut pos = 7;

    while current_byte >= 128 {
        current_byte = any.parse_next(input)?;

        // these are the bits that constitute the size, which is the byte as-is, but with the first
        // bit masked off as it's the continuation bit
        let chunk = (current_byte & 0b0111_1111) as u64;

        value |= chunk << pos;
        pos += 7;
    }

    Ok(value)
}
