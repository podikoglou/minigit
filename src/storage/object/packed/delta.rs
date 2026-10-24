use winnow::{
    ModalResult, Parser,
    combinator::{alt, repeat},
    error::StrContext,
    token::{any, take},
};

use crate::{
    MinigitError,
    object::hash::parse_object_hash,
    parsing::Stream,
    storage::object::{
        loose::Encode,
        packed::{BaseObject, PackedObjectType, offset, size},
    },
};

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Insert(InsertInstruction),
    Copy(CopyInstruction),
}

impl Encode for Instruction {
    fn encode<W: std::io::prelude::Write>(&self, writer: &mut W) -> Result<(), MinigitError> {
        match self {
            Instruction::Insert(instruction) => instruction.encode(writer),
            Instruction::Copy(instruction) => instruction.encode(writer),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct InsertInstruction(pub Vec<u8>);

impl Encode for InsertInstruction {
    fn encode<W: std::io::prelude::Write>(&self, writer: &mut W) -> Result<(), MinigitError> {
        let len = self.0.len();

        // if len is bigger than 128 (0b0000_0000), then we can't encode it, because the MSB should
        // be 0 to signify that this is an insert instruction
        //
        // (if it is larger than 128 the data should be split in different instructions)
        //
        // TODO: don't check this here, enforce it during the creation of `InsertInstruction`
        if len == 0 || len >= 128 {
            return Err(MinigitError::InvalidInstruction);
        }

        writer.write_all(&[len as u8])?;
        writer.write_all(&self.0)?;

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct CopyInstruction {
    pub offset: u64,
    pub size: u64,
}

impl Encode for CopyInstruction {
    fn encode<W: std::io::prelude::Write>(&self, writer: &mut W) -> Result<(), MinigitError> {
        let CopyInstruction { offset, size } = *self;

        // amount of bytes the offset will occupy
        let offset_bytes = {
            if offset > 0xFFFFFF {
                4
            } else if offset > 0xFFFF {
                3
            } else if offset > 0xFF {
                2
            } else {
                1
            }
        };

        let size_bytes = {
            if size > 0xFFFFFF {
                // size can only occupy up to 3 bytes
                return Err(MinigitError::InvalidInstruction);
            } else if size > 0xFFFF {
                3
            } else if size > 0xFF {
                2
            } else {
                1
            }
        };

        let mut first: u8 = 0b1000_0000;

        match offset_bytes {
            4 => first |= 0b0000_1111,
            3 => first |= 0b0000_0111,
            2 => first |= 0b0000_0011,
            1 => first |= 0b0000_0001,
            _ => unreachable!(),
        };

        match size_bytes {
            3 => first |= 0b0111_0000,
            2 => first |= 0b0011_0000,
            1 => first |= 0b0001_0000,
            _ => unreachable!(),
        };

        writer.write_all(&[first])?;

        let offset_le = offset.to_le_bytes();

        match offset_bytes {
            1 => writer.write(&offset_le[0..=0]),
            2 => writer.write(&offset_le[0..=1]),
            3 => writer.write(&offset_le[0..=2]),
            4 => writer.write(&offset_le[0..=3]),
            _ => unreachable!(),
        }?;

        let size_le = size.to_le_bytes();

        match size_bytes {
            1 => writer.write(&size_le[0..=0]),
            2 => writer.write(&size_le[0..=1]),
            3 => writer.write(&size_le[0..=2]),

            _ => unreachable!(),
        }?;

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct Delta {
    pub base_object_size: u64,
    pub deltified_object_size: u64,
    pub instructions: Vec<Instruction>,
}

impl Encode for Delta {
    fn encode<W: std::io::prelude::Write>(&self, writer: &mut W) -> Result<(), MinigitError> {
        (self.base_object_size as usize).encode(writer)?;
        (self.deltified_object_size as usize).encode(writer)?;

        for instruction in &self.instructions {
            instruction.encode(writer)?;
        }

        Ok(())
    }
}

/// Implements Git's size encoding
impl Encode for usize {
    fn encode<W: std::io::prelude::Write>(&self, writer: &mut W) -> Result<(), MinigitError> {
        let mut remaining = *self;

        while remaining >= 0b1000_0000 {
            let byte = (remaining as u8) | 0b1000_0000;

            writer.write_all(&[byte])?;
            remaining >>= 7;
        }
        let byte = remaining as u8;

        writer.write_all(&[byte])?;

        Ok(())
    }
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

pub fn insert_instruction(input: &mut Stream<'_>) -> ModalResult<InsertInstruction> {
    let size = any
        .verify(|byte| (1..128).contains(byte))
        .parse_next(input)?;

    take(size as usize)
        .map(|x: &[u8]| InsertInstruction(x.into()))
        .parse_next(input)
}

pub fn copy_instruction(input: &mut Stream<'_>) -> ModalResult<CopyInstruction> {
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
