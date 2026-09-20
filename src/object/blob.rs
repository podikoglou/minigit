use std::io::Write;

use winnow::{ModalResult, Parser, error::StrContext, token::rest};

use crate::{parsing::Stream, storage::object::loose::WriteLoose};

/// An blob: an object that simply contains some bytes.
///
/// [trees](`super::Tree`) refer to blobs, usually.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Blob(pub Vec<u8>);

impl Blob {
    #[must_use]
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl WriteLoose for Blob {
    fn write_loose<W: Write>(&self, writer: &mut W) -> Result<(), crate::MinigitError> {
        writer.write_all(&self.0)?;
        Ok(())
    }
}

/// Parses a blob object's content from some bytes.
pub fn parse_blob<'a>(input: &mut Stream<'a>) -> ModalResult<Blob> {
    rest.map(|e: Stream| Blob(e.into()))
        .context(StrContext::Label("blob object"))
        .parse_next(input)
}
