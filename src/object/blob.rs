use std::io::Write;

use winnow::{ModalResult, Parser, error::StrContext, token::rest};

use crate::{ende::Decode, parsing::Stream, ende::Encode};

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

impl Decode for Blob {
    fn parse(input: &mut Stream<'_>) -> ModalResult<Self> {
        rest.map(|e: &[u8]| Blob(e.into()))
            .context(StrContext::Label("blob object"))
            .parse_next(input)
    }
}

impl Encode for Blob {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), crate::TinygitError> {
        writer.write_all(&self.0)?;
        Ok(())
    }
}
