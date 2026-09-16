use std::io::Write;

use crate::storage::object::loose::WriteLoose;

/// An blob: an object that simply contains some bytes.
///
/// [trees](`super::Tree`) refer to blobs, usually.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Blob(pub Vec<u8>);

impl Blob {
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
