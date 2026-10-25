//! This module deals with packed objects.
//!
//! The main types are [DeltifiedObject], [UndeltifiedObject], and [PackedObject] that holds one of
//! the two.
use std::io::Read;

use flate2::read::ZlibDecoder;
use nutype::nutype;
use strum::FromRepr;
use winnow::{
    ModalResult, Parser,
    combinator::alt,
    error::{ContextError, ErrMode, StrContext},
    stream::Location,
    token::{any, take},
};

use crate::{
    MinigitError,
    object::{ObjectType, hash::ObjectHash},
    parsing::Stream,
    storage::object::{
        RawObject,
        loose::Encode,
        packed::{
            base_object,
            delta::{Delta, delta},
        },
    },
};

#[derive(Debug, PartialEq)]
pub struct PackedObjectHeader {
    /// The object type.
    pub r#type: PackedObjectType,

    /// The length of the *decompressed* data.
    pub length: u64,
}

impl Encode for PackedObjectHeader {
    fn encode<W: std::io::prelude::Write>(&self, writer: &mut W) -> Result<(), MinigitError> {
        let type_id = self.r#type as u8;

        let mut value = self.length;

        let mut curr_byte = type_id << 4 | (value & 0x0F) as u8;
        value >>= 4;

        if value != 0 {
            curr_byte |= 1 << 7;
        }

        writer.write_all(&[curr_byte])?;

        while value != 0 {
            curr_byte = (value & 0x7F) as u8;
            value >>= 7;

            if value != 0 {
                curr_byte |= 1 << 7;
            }

            writer.write_all(&[curr_byte])?;
        }

        Ok(())
    }
}

#[derive(FromRepr, Debug, PartialEq, Clone, Copy)]
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

#[nutype(derive(Debug, PartialEq))]
pub struct DeltifiedObject((BaseObject, Delta));

#[nutype(derive(Debug, PartialEq))]
pub struct UndeltifiedObject(RawObject);

impl Encode for BaseObject {
    fn encode<W: std::io::prelude::Write>(&self, writer: &mut W) -> Result<(), MinigitError> {
        match self {
            BaseObject::Ref(hash) => {
                let bytes: &[u8] = hash.into();

                writer.write_all(bytes)?;
            }
            BaseObject::Ofs(offset) => {
                let mut num = *offset;
                let mut buf = [0u8; 10];
                let mut pos = buf.len();

                pos -= 1;
                buf[pos] = (num & 0x7F) as u8;

                while num > 0x7F {
                    num = (num >> 7) - 1;
                    pos -= 1;
                    buf[pos] = 0x80 | ((num & 0x7F) as u8);
                }

                writer.write_all(&buf[pos..])?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum PackedObject {
    Deltified(usize, DeltifiedObject),
    Undeltified(usize, UndeltifiedObject),
}

impl PackedObject {
    /// Gets the offset of the object in the packfile.
    pub fn offset(&self) -> usize {
        match self {
            PackedObject::Deltified(offset, _) => *offset,
            PackedObject::Undeltified(offset, _) => *offset,
        }
    }
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

    // keep reading, the continuation bit is 1
    // (thanks for the trick Aditya! - https://codewords.recurse.com/issues/three/unpacking-git-packfiles)
    while current_byte >= 128 && pos < 67 {
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
    let offset = input.current_token_start();

    alt((
        deltified_object
            .map(|object| PackedObject::Deltified(offset, object))
            .context(StrContext::Label("deltified object")),
        undeltified_object
            .map(|object| PackedObject::Undeltified(offset, object))
            .context(StrContext::Label("undeltified object")),
    ))
    .context(StrContext::Label("object"))
    .parse_next(input)
}

/// Parses the header and data of a deltified object. Does not take care of resolving the deltas.
fn deltified_object(input: &mut Stream<'_>) -> ModalResult<DeltifiedObject> {
    let offset = input.current_token_start();

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
        .map(|base| {
            match base {
                BaseObject::Ofs(relative_offset) => {
                    // we resolve the relative offset ofs_delta gives us offset to a file-level offset
                    BaseObject::Ofs(offset as u64 - relative_offset)
                }

                other => other,
            }
        })
        .parse_next(input)?;

    let mut buf = vec![0; header.length as usize];
    let mut decoder = ZlibDecoder::new(&input[..]);

    decoder
        .read_exact(&mut buf)
        .map_err(|_| ErrMode::Cut(ContextError::new()))?;

    // consume the amount of bytes that the decoder read, advancing the `input` slice
    take(decoder.total_in() as usize).parse_next(input)?;

    let delta = delta.parse_next(&mut Stream::new(&buf))?;

    Ok(DeltifiedObject::new((base_object, delta)))
}

/// Parses the header and data of an undeltified object. Also takes care of decompressing the data.
pub fn undeltified_object(input: &mut Stream<'_>) -> ModalResult<RawObject> {
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

    if header.length == 0 {
        // fixed-sized zlib header + trailer
        // not sure why but it's not cosnsumed with the below code
        take(8usize).parse_next(input)?;
    } else {
        let mut decoder = ZlibDecoder::new(&input[..]);

        decoder
            .read_exact(&mut buf)
            .map_err(|_| ErrMode::Cut(ContextError::new()))?;

        // consume the amount of bytes that the decoder read, advancing the `input` slice
        take(decoder.total_in() as usize).parse_next(input)?;
    }

    let r#type = match header.r#type {
        PackedObjectType::Commit => ObjectType::Commit,
        PackedObjectType::Tree => ObjectType::Tree,
        PackedObjectType::Blob => ObjectType::Blob,
        PackedObjectType::Tag => ObjectType::Tag,

        _ => unreachable!(),
    };

    Ok(UndeltifiedObject::new(RawObject::new(r#type, buf)))
}

#[cfg(test)]
mod tests {
    use crate::{
        object::ObjectType,
        parsing::Stream,
        storage::object::{
            RawObject,
            packed::{UndeltifiedObject, object::undeltified_object},
        },
    };

    use super::{BaseObject, PackedObjectType, base_object, object_header};
    use insta::assert_debug_snapshot;
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

        assert_debug_snapshot!(obj);
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
