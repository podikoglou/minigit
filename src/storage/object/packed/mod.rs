//! This module deals with packfiles
//!
//! References:
//! - <https://git-scm.com/docs/pack-format>
//! - <https://git-scm.com/book/en/v2/Git-Internals-Packfiles>
//!
//! In particular, this deals with version 2 of the packfile format.

pub mod idx;

use std::{fs::File, io::Read, path::PathBuf};

use flate2::read::ZlibDecoder;
use memmap2::Mmap;
use strum::FromRepr;
use winnow::{
    ModalResult, Parser,
    binary::be_u32,
    combinator::{alt, repeat, seq},
    error::{ContextError, ErrMode, StrContext},
    token::{any, take},
};

use crate::{
    MinigitError,
    error::ParserContext,
    object::{
        Object,
        blob::parse_blob,
        commit::parse_commit,
        hash::{ObjectHash, parse_object_hash},
        tag::parse_tag,
        tree::parse_tree,
    },
    parsing::Stream,
};

/// Holds a handle to an memory-mapped packfie and optionally its index.
#[derive(Debug)]
pub struct Packfile {
    pub pack: (PathBuf, Mmap),
    pub idx: Option<(PathBuf, Mmap)>,

    /// The amount of objects contained in the packfile.
    pub objects_count: u32,
}

impl Packfile {
    /// Creates an instance of [Packfile]. This memory maps the pack and (if present) index file,
    /// and parses their headers.
    pub fn open(pack_path: PathBuf, idx_path: Option<PathBuf>) -> Result<Self, MinigitError> {
        // open packfile
        let pack_file = File::open(&pack_path)?;
        let pack_buf = unsafe { Mmap::map(&pack_file) }?;

        let pack = (pack_path, pack_buf);

        // read packfile header
        let pack_header = header.parse(&mut &pack.1[..12]).map_err(|err| {
            MinigitError::ParserError(err.to_string(), ParserContext::File(pack.0.clone()))
        })?;

        // if present, open idx
        let idx = if let Some(path) = idx_path {
            let idx_file = File::open(&path)?;
            let idx_buf = unsafe { Mmap::map(&idx_file) }?;

            let idx = (path, idx_buf);

            // read idx header
            //
            // this return any information, this is mostly for validation that this is a valid idx
            // file.
            idx::header.parse(&mut &idx.1[..12]).map_err(|err| {
                MinigitError::ParserError(err.to_string(), ParserContext::File(pack.0.clone()))
            })?;

            Some(idx)
        } else {
            None
        };

        Ok(Self {
            pack,
            idx,
            objects_count: pack_header.objects,
        })
    }

    /// Returns an iterator over the objects of the packfile.
    pub fn objects<'a>(&'a self) -> Objects<'a> {
        Objects {
            buf: &self.pack.1[12..],
        }
    }

    /// Reads an object at a specific offset of the packfile and returns it.
    ///
    /// This does not resolve the deltas.
    pub fn read_object_at_offset(&self, offset: usize) -> Result<PackedObject, MinigitError> {
        object
            .parse_next(&mut &self.pack.1[offset..])
            .map_err(|err| {
                MinigitError::ParserError(err.to_string(), ParserContext::File(self.pack.0.clone()))
            })
    }
}

impl PartialEq for Packfile {
    fn eq(&self, other: &Self) -> bool {
        self.pack.0 == other.pack.0
            && self.idx.as_ref().map(|(x, _)| x) == other.idx.as_ref().map(|(x, _)| x)
            && self.objects_count == other.objects_count
    }
}

/// An iterator over [PackedObject].
pub struct Objects<'a> {
    /// A slice of the packfile's contents, starting from the first object.
    buf: &'a [u8],
}

impl Iterator for Objects<'_> {
    type Item = PackedObject;

    fn next(&mut self) -> Option<Self::Item> {
        object.parse_next(&mut self.buf).ok()
    }
}
impl Eq for Packfile {}

#[derive(Debug)]
pub struct PackfileHeader {
    /// The amount of objects contained in the packfile.
    pub objects: u32,
}

#[derive(Debug)]
pub struct PackedObjectHeader {
    /// The object type.
    pub r#type: PackedObjectType,

    /// The length of the *decompressed* data.
    pub length: u64,
}

#[derive(FromRepr, Debug, PartialEq)]
#[repr(u8)]
pub enum PackedObjectType {
    Commit = 1,
    Tree = 2,
    Blob = 3,
    Tag = 4,
    OfsDelta = 6,
    RefDelta = 7,
}

#[derive(Debug, PartialEq)]
pub enum BaseObject {
    Ref(ObjectHash),

    Ofs(u64),
}

#[derive(Debug, PartialEq)]
pub enum PackedObject {
    Deltified { base: BaseObject, delta: Delta },
    Undeltified(Object),
}

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Insert(InsertInstruction),
    Copy(CopyInstruction),
}

#[derive(Debug, PartialEq)]
pub struct InsertInstruction(pub Vec<u8>);

#[derive(Debug, PartialEq)]
pub struct CopyInstruction {
    pub offset: u64,
    pub size: u64,
}

#[derive(Debug, PartialEq)]
pub struct Delta {
    pub base_object_size: u64,
    pub deltified_object_size: u64,
    pub instructions: Vec<Instruction>,
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

pub fn object_header(input: &mut Stream<'_>) -> ModalResult<PackedObjectHeader> {
    // TODO: using combinators instead of doing this imperatively would be great.

    let mut current_byte = any.parse_next(input)?;

    let type_id = (current_byte & 0b01110000) >> 4;

    // TODO: better error handling
    let r#type =
        PackedObjectType::from_repr(type_id).ok_or_else(|| ErrMode::Cut(ContextError::new()))?;

    let mut size = (current_byte & 0b00001111) as u64;
    let mut pos = 4;

    // TODO: inside this loop, we must check the size will fit inside a u64
    // libgit2 does something similar:
    // https://github.com/libgit2/libgit2/blob/0551dfd4ad989b6a3d5683c0d4cf326c6efef929/src/libgit2/pack.c#L434

    // keep reading, the continuation bit is 1
    // (thanks for the trick Aditya! - https://codewords.recurse.com/issues/three/unpacking-git-packfiles)
    while current_byte >= 128 {
        current_byte = any.parse_next(input)?;

        // these are the bits that constitute the size, which is the byte as-is, but with the first
        // bit masked off as it's the continuation bit
        let chunk = (current_byte & 0b0111_1111) as u64;

        size |= chunk << pos;
        pos += 7;
    }

    Ok(PackedObjectHeader {
        r#type,
        length: size,
    })
}

/// Parses an object. Takes care of parsing the header too.
pub fn object(input: &mut Stream<'_>) -> ModalResult<PackedObject> {
    alt((
        deltified_object
            .map(|(base, delta)| PackedObject::Deltified { base, delta })
            .context(StrContext::Label("deltified object")),
        undeltified_object
            .map(PackedObject::Undeltified)
            .context(StrContext::Label("undeltified object")),
    ))
    .context(StrContext::Label("object"))
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

fn base_object(r#type: PackedObjectType) -> impl FnMut(&mut Stream<'_>) -> ModalResult<BaseObject> {
    match r#type {
        PackedObjectType::OfsDelta => |input: &mut Stream<'_>| {
            offset
                .map(BaseObject::Ofs)
                .context(StrContext::Label("base object offset"))
                .parse_next(input)
        },
        PackedObjectType::RefDelta => |input: &mut Stream<'_>| {
            parse_object_hash
                .map(BaseObject::Ref)
                .context(StrContext::Label("base object name"))
                .parse_next(input)
        },

        _ => unreachable!(),
    }
}

/// Parses the header and data of a deltified object. Does not take care of resolving the deltas.
fn deltified_object(input: &mut Stream<'_>) -> ModalResult<(BaseObject, Delta)> {
    let header = object_header
        .verify(|header| {
            matches!(
                header.r#type,
                PackedObjectType::OfsDelta | PackedObjectType::RefDelta
            )
        })
        .context(StrContext::Label("object header"))
        .parse_next(input)?;

    let base_object = base_object(header.r#type)
        .context(StrContext::Label("base object"))
        .parse_next(input)?;

    let mut buf = vec![0; header.length as usize];
    let mut decoder = ZlibDecoder::new(&input[..]);

    decoder
        .read_exact(&mut buf)
        .map_err(|_| ErrMode::Cut(ContextError::new()))?;

    // consume the amount of bytes that the decoder read, advancing the `input` slice
    take(decoder.total_in() as usize).parse_next(input)?;

    let delta = delta.parse_next(&mut Stream::new(&buf))?;

    Ok((base_object, delta))
}

fn size(input: &mut Stream<'_>) -> ModalResult<u64> {
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

fn delta(input: &mut Stream<'_>) -> ModalResult<Delta> {
    let base_object_size = size
        .context(StrContext::Label("base object size"))
        .parse_next(input)?;

    let deltified_object_size = size
        .context(StrContext::Label("base object size"))
        .parse_next(input)?;

    let instructions = repeat(0.., instruction).parse_next(input)?;

    Ok(Delta {
        base_object_size,
        deltified_object_size,
        instructions,
    })
}

fn instruction(input: &mut Stream<'_>) -> ModalResult<Instruction> {
    alt((
        insert_instruction.map(Instruction::Insert),
        copy_instruction.map(Instruction::Copy),
    ))
    .parse_next(input)
}

fn insert_instruction(input: &mut Stream<'_>) -> ModalResult<InsertInstruction> {
    let first_byte = any.verify(|byte| *byte < 128).parse_next(input)?;
    let size = first_byte & 0b0111_1111;

    take(size as usize)
        .map(|x: &[u8]| InsertInstruction(x.into()))
        .parse_next(input)
}

fn copy_instruction(input: &mut Stream<'_>) -> ModalResult<CopyInstruction> {
    let first_byte = any.verify(|byte| *byte >= 128).parse_next(input)?;

    let has_offset_1 = (first_byte & 0b0000_0001) != 0;
    let has_offset_2 = (first_byte & 0b0000_0010) != 0;
    let has_offset_3 = (first_byte & 0b0000_0100) != 0;
    let has_offset_4 = (first_byte & 0b0000_1000) != 0;
    let has_size_1 = (first_byte & 0b0001_0000) != 0;
    let has_size_2 = (first_byte & 0b0010_0000) != 0;
    let has_size_3 = (first_byte & 0b0100_0000) != 0;

    let offset_1 = if has_offset_1 {
        any.parse_next(input)? as u64
    } else {
        0
    };

    let offset_2 = if has_offset_2 {
        any.parse_next(input)? as u64
    } else {
        0
    };

    let offset_3 = if has_offset_3 {
        any.parse_next(input)? as u64
    } else {
        0
    };

    let offset_4 = if has_offset_4 {
        any.parse_next(input)? as u64
    } else {
        0
    };

    let size_1 = if has_size_1 {
        any.parse_next(input)? as u64
    } else {
        0
    };
    let size_2 = if has_size_2 {
        any.parse_next(input)? as u64
    } else {
        0
    };
    let size_3 = if has_size_3 {
        any.parse_next(input)? as u64
    } else {
        0
    };

    let offset = offset_1 | (offset_2 << 8) | (offset_3 << 16) | (offset_4 << 24);
    let size = size_1 | (size_2 << 8) | (size_3 << 16);

    Ok(CopyInstruction { offset, size })
}

/// Parses the header and data of an undeltified object. Also takes care of decompressing the data.
fn undeltified_object(input: &mut Stream<'_>) -> ModalResult<Object> {
    let header = object_header
        .verify(|header| {
            matches!(
                header.r#type,
                PackedObjectType::Commit
                    | PackedObjectType::Tag
                    | PackedObjectType::Blob
                    | PackedObjectType::Tree
            )
        })
        .context(StrContext::Label("object header"))
        .parse_next(input)?;

    // we create a buffer with the size we read from the header
    //
    // keep in mind the fact that this size is the size of the decompressed data
    // rather than the compressed data
    let mut buf = vec![0; header.length as usize];
    let mut decoder = ZlibDecoder::new(&input[..]);

    decoder
        .read_exact(&mut buf)
        .map_err(|_| ErrMode::Cut(ContextError::new()))?;

    // `input`, as of the below line, should point to he beginning of the next object's header.
    //
    // because we don't use winnow for parsing the zlib-compressed data, the `input` slice is never
    // advanced to indicate the position in the input.
    //
    // this line replaces the input with the decoder's `inner` field (which is the same slice we
    // passed to it when we created it), which *is* advanced.
    //
    // this is only useful for when the caller of this function needs to linearly read the packfile
    take(decoder.total_in() as usize).parse_next(input)?;

    match header.r#type {
        PackedObjectType::Commit => parse_commit
            .map(Object::from)
            .parse_next(&mut Stream::new(&buf)),

        PackedObjectType::Tree => parse_tree
            .map(Object::from)
            .parse_next(&mut Stream::new(&buf)),

        PackedObjectType::Blob => parse_blob
            .map(Object::from)
            .parse_next(&mut Stream::new(&buf)),

        PackedObjectType::Tag => parse_tag
            .map(Object::from)
            .parse_next(&mut Stream::new(&buf)),

        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        object::Object,
        parsing::Stream,
        storage::object::packed::undeltified_object,
    };

    use super::{BaseObject, PackedObjectType, base_object, object_header};
    use std::assert_matches;
    use winnow::Parser;

    #[test]
    fn object_header_parses_basic_headers() {
        let (rest, header) = object_header
            .parse_peek(Stream::new(&[0x99, 0x0a, 0x78, 0x9c][..]))
            .expect("commit header should parse");
        assert_eq!(header.r#type, PackedObjectType::Commit);
        assert_eq!(header.length, 169);
        assert_eq!(*rest, &[0x78, 0x9c]);

        let (rest, header) = object_header
            .parse_peek(Stream::new(&[0xb7, 0x02, 0x78, 0x9c][..]))
            .expect("blob header should parse");
        assert_eq!(header.r#type, PackedObjectType::Blob);
        assert_eq!(header.length, 39);
        assert_eq!(*rest, &[0x78, 0x9c]);

        let (rest, header) = object_header
            .parse_peek(Stream::new(&[0xa4, 0x02, 0x78, 0x9c][..]))
            .expect("tree header should parse");
        assert_eq!(header.r#type, PackedObjectType::Tree);
        assert_eq!(header.length, 36);
        assert_eq!(*rest, &[0x78, 0x9c]);
    }

    #[test]
    fn object_header_parses_single_byte_header() {
        let (rest, header) = object_header
            .parse_peek(Stream::new(&[0x19, 0x0a][..]))
            .expect("commit header should parse");
        assert_eq!(header.r#type, PackedObjectType::Commit);
        assert_eq!(header.length, 9);
        assert_eq!(*rest, &[0x0a]);
    }

    #[test]
    fn undeltified_object_parses_basic_object() {
        let (_, obj) = undeltified_object
            .parse_peek(Stream::new(&[
                0x99, 0x0a, 0x78, 0x9c, 0x9d, 0xcb, 0x4d, 0x0a, 0xc2, 0x30, 0x10, 0x40, 0xe1, 0x7d,
                0x4e, 0x31, 0x7b, 0xa1, 0x64, 0x12, 0xf3, 0x53, 0x10, 0xf1, 0x2a, 0x99, 0x66, 0xd2,
                0x0e, 0x26, 0x46, 0x4a, 0x0a, 0x1e, 0x5f, 0xbd, 0x42, 0x37, 0x6f, 0xf1, 0xc1, 0x1b,
                0x3b, 0x33, 0xc4, 0x84, 0x3e, 0x73, 0x28, 0x14, 0x09, 0x83, 0x8f, 0x64, 0x9c, 0x0b,
                0x4b, 0xd1, 0x96, 0x8c, 0x29, 0xd1, 0x73, 0x66, 0x6b, 0x08, 0x73, 0xc9, 0x2a, 0x1d,
                0x63, 0xeb, 0x3b, 0xa4, 0xca, 0x1f, 0xb8, 0xfd, 0x3b, 0xbd, 0x7b, 0x96, 0x67, 0x5f,
                0x6b, 0x3f, 0x1e, 0x6b, 0x4b, 0x52, 0xa7, 0xa5, 0xb7, 0x3b, 0x60, 0x88, 0xb3, 0xf7,
                0xd7, 0x19, 0x1d, 0x5c, 0xb4, 0xd5, 0x5a, 0xfd, 0xb4, 0xc9, 0x18, 0x7c, 0xe6, 0x55,
                0xf2, 0x92, 0xa1, 0xbe, 0xc5, 0xe5, 0x34, 0x91, 0xb7, 0x02, 0x78, 0x9c,
            ]))
            .expect("commit object should parse");

        assert_matches!(obj, Object::Commit(_));
    }

    #[test]
    fn parse_base_object_parses_offset_delta() {
        let (rest, base_object) = base_object(PackedObjectType::OfsDelta)
            .parse_peek(Stream::new(&[0x81, 0x08, 0x78, 0x9c][..]))
            .expect("ofs delta base object should parse");

        assert_eq!(base_object, BaseObject::Ofs(264));
        assert_eq!(*rest, &[0x78, 0x9c]);
    }
}
