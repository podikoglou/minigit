use std::{
    fs,
    io::{BufReader, Read},
    path::PathBuf,
    rc::Rc,
};

use crate::{
    MinigitError,
    error::ParserContext,
    object::{Object, hash::ObjectHash, parse_header},
    parsing::Stream,
    storage::object::{RawObject, loose, packed::Packfile},
};

use flate2::bufread::ZlibDecoder;
use winnow::{
    Parser,
    error::{ContextError, ErrMode},
    token::take,
};

/// An object which has not been loaded yet.
///
/// At any given time it can be turned into a real [Object] using [`Self::into_object`].
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LazyObject {
    Loose(PathBuf),
    Packed(Rc<Packfile>, usize),
}

impl LazyObject {
    #[must_use]
    pub fn loose(path: PathBuf) -> Self {
        Self::Loose(path)
    }

    /// Reads and decompresses the object into a [RawObject] container.
    pub fn into_raw(&self) -> Result<RawObject, MinigitError> {
        match self {
            Self::Loose(path) => {
                let contents = fs::read(path)?;

                // decompress file contents
                let mut decoder = ZlibDecoder::new(&contents[..]);

                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;

                // parse just the header of the object
                let mut stream = Stream::new(&decompressed);

                let (r#type, size) = parse_header.parse_next(&mut stream).map_err(|err| {
                    MinigitError::ParserError(err.to_string(), ParserContext::File(path.clone()))
                })?;

                // take the object's payload verbatim
                // NOTE: PERF: This creates a new Vec<u8>, so we have duplicate data in memory
                // (`decompressed`, and `object_bytes`).
                let object_bytes: Vec<u8> = take(size)
                    .parse_next(&mut stream)
                    .map_err(|err: ErrMode<ContextError>| {
                        MinigitError::ParserError(
                            err.to_string(),
                            ParserContext::File(path.clone()),
                        )
                    })?
                    .into();

                Ok(RawObject::new(r#type, object_bytes))
            }
            Self::Packed(packfile, offset) => {
                let packed_obj = packfile.read_object_at_offset(*offset)?;

                packfile.resolve(packed_obj)
            }
        }
    }

    /// Reads and parses the full object.
    pub fn into_object(&self) -> Result<Object, MinigitError> {
        let raw = self.into_raw()?;

        raw.into_object()
    }

    pub fn hash(&self) -> Result<ObjectHash, MinigitError> {
        match self {
            Self::Loose(path) => path.try_into(),
            Self::Packed(packfile, offset) => todo!(),
        }
    }
}
