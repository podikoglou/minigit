//! This module deals with packfiles
//!
//! References:
//! - <https://git-scm.com/docs/pack-format>
//! - <https://git-scm.com/book/en/v2/Git-Internals-Packfiles>
//!
//! In particular, this deals with version 2 of the packfile format.

mod idx;

use std::io::Read;

use flate2::read::ZlibDecoder;
use strum::FromRepr;
use winnow::{
    ModalResult, Parser,
    binary::be_u32,
    combinator::seq,
    error::{ContextError, ErrMode, StrContext},
    token::any,
};

use crate::{
    object::{
        Object,
        blob::parse_blob,
        commit::parse_commit,
        hash::{ObjectHash, parse_object_hash_str},
        tag::parse_tag,
        tree::parse_tree,
    },
    parsing::Stream,
};

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
            parse_object_hash_str
                .map(BaseObject::Ref)
                .context(StrContext::Label("base object name"))
                .parse_next(input)
        },

        _ => unreachable!(),
    }
}

/// Parses the header and data of a deltified object. Does not take care of resolving the deltas.
fn deltified_object(input: &mut Stream<'_>) -> ModalResult<()> {
    let header = object_header
        .verify(|header| {
            matches!(
                header.r#type,
                PackedObjectType::OfsDelta | PackedObjectType::RefDelta
            )
        })
        .parse_next(input)?;

    let base_object = base_object(header.r#type)
        .context(StrContext::Label("base object"))
        .parse_next(input)?;

    todo!()
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
    *input = decoder.into_inner();

    match header.r#type {
        PackedObjectType::Commit => parse_commit
            .map(Object::from)
            .parse_next(&mut buf.as_slice()),

        PackedObjectType::Tree => parse_tree.map(Object::from).parse_next(&mut buf.as_slice()),
        PackedObjectType::Blob => parse_blob.map(Object::from).parse_next(&mut buf.as_slice()),
        PackedObjectType::Tag => parse_tag.map(Object::from).parse_next(&mut buf.as_slice()),

        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{object::Object, storage::object::packed::undeltified_object};

    use super::{BaseObject, PackedObjectType, base_object, object_header};
    use std::assert_matches;
    use winnow::Parser;

    #[test]
    fn object_header_parses_basic_headers() {
        let (rest, header) = object_header
            .parse_peek(&[0x99, 0x0a, 0x78, 0x9c][..])
            .expect("commit header should parse");
        assert_eq!(header.r#type, PackedObjectType::Commit);
        assert_eq!(header.length, 169);
        assert_eq!(rest, &[0x78, 0x9c]);

        let (rest, header) = object_header
            .parse_peek(&[0xb7, 0x02, 0x78, 0x9c][..])
            .expect("blob header should parse");
        assert_eq!(header.r#type, PackedObjectType::Blob);
        assert_eq!(header.length, 39);
        assert_eq!(rest, &[0x78, 0x9c]);

        let (rest, header) = object_header
            .parse_peek(&[0xa4, 0x02, 0x78, 0x9c][..])
            .expect("tree header should parse");
        assert_eq!(header.r#type, PackedObjectType::Tree);
        assert_eq!(header.length, 36);
        assert_eq!(rest, &[0x78, 0x9c]);
    }

    #[test]
    fn object_header_parses_single_byte_header() {
        let (rest, header) = object_header
            .parse_peek(&[0x19, 0x0a][..])
            .expect("commit header should parse");
        assert_eq!(header.r#type, PackedObjectType::Commit);
        assert_eq!(header.length, 9);
        assert_eq!(rest, &[0x0a]);
    }

    #[test]
    fn undeltified_object_parses_basic_object() {
        let (_, obj) = undeltified_object
            .parse_peek(&[
                0x99, 0x0a, 0x78, 0x9c, 0x9d, 0xcb, 0x4d, 0x0a, 0xc2, 0x30, 0x10, 0x40, 0xe1, 0x7d,
                0x4e, 0x31, 0x7b, 0xa1, 0x64, 0x12, 0xf3, 0x53, 0x10, 0xf1, 0x2a, 0x99, 0x66, 0xd2,
                0x0e, 0x26, 0x46, 0x4a, 0x0a, 0x1e, 0x5f, 0xbd, 0x42, 0x37, 0x6f, 0xf1, 0xc1, 0x1b,
                0x3b, 0x33, 0xc4, 0x84, 0x3e, 0x73, 0x28, 0x14, 0x09, 0x83, 0x8f, 0x64, 0x9c, 0x0b,
                0x4b, 0xd1, 0x96, 0x8c, 0x29, 0xd1, 0x73, 0x66, 0x6b, 0x08, 0x73, 0xc9, 0x2a, 0x1d,
                0x63, 0xeb, 0x3b, 0xa4, 0xca, 0x1f, 0xb8, 0xfd, 0x3b, 0xbd, 0x7b, 0x96, 0x67, 0x5f,
                0x6b, 0x3f, 0x1e, 0x6b, 0x4b, 0x52, 0xa7, 0xa5, 0xb7, 0x3b, 0x60, 0x88, 0xb3, 0xf7,
                0xd7, 0x19, 0x1d, 0x5c, 0xb4, 0xd5, 0x5a, 0xfd, 0xb4, 0xc9, 0x18, 0x7c, 0xe6, 0x55,
                0xf2, 0x92, 0xa1, 0xbe, 0xc5, 0xe5, 0x34, 0x91, 0xb7, 0x02, 0x78, 0x9c,
            ])
            .expect("commit object should parse");

        assert_matches!(obj, Object::Commit(_));
    }

    #[test]
    fn parse_base_object_parses_offset_delta() {
        let (rest, base_object) = base_object(PackedObjectType::OfsDelta)
            .parse_peek(&[0x81, 0x08, 0x78, 0x9c][..])
            .expect("ofs delta base object should parse");

        assert_eq!(base_object, BaseObject::Ofs(264));
        assert_eq!(rest, &[0x78, 0x9c]);
    }
}
