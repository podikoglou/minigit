use winnow::{
    ModalResult, Parser,
    combinator::{alt, repeat},
    error::StrContext,
    token::{any, take},
};

use crate::{
    object::hash::parse_object_hash,
    parsing::Stream,
    storage::object::packed::{BaseObject, PackedObjectType, offset, size},
};

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

pub fn base_object(
    r#type: PackedObjectType,
) -> impl FnMut(&mut Stream<'_>) -> ModalResult<BaseObject> {
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

pub fn delta(input: &mut Stream<'_>) -> ModalResult<Delta> {
    let base_object_size = size
        .context(StrContext::Label("base object size"))
        .parse_next(input)?;

    let deltified_object_size = size
        .context(StrContext::Label("deltified object size"))
        .parse_next(input)?;

    let instructions = repeat(0.., instruction).parse_next(input)?;

    Ok(Delta {
        base_object_size,
        deltified_object_size,
        instructions,
    })
}

pub fn instruction(input: &mut Stream<'_>) -> ModalResult<Instruction> {
    alt((
        insert_instruction.map(Instruction::Insert),
        copy_instruction.map(Instruction::Copy),
    ))
    .parse_next(input)
}

fn insert_instruction(input: &mut Stream<'_>) -> ModalResult<InsertInstruction> {
    let size = any.verify(|byte| *byte < 128).parse_next(input)?;

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
    let size = if size == 0 { 0x10000 } else { size };

    Ok(CopyInstruction { offset, size })
}
