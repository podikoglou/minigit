use std::io;

use hex::FromHexError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MinigitError {
    #[error("IO Error: {0}")]
    IO(#[from] io::Error),

    #[error("Invalid file name")]
    InvalidFileName,

    #[error("No git directory")]
    NoGitDirectory,

    #[error("Bucket not found")]
    BucketNotFound,

    #[error("Object not found")]
    ObjectNotFound,

    // Ideally, we'd take advante of winnow's `ParseError`, but it contains a lifetime and we'd need
    // to make this type something like `MinigitError<'a>`, which means we'd need to specify the
    // lifetime every time we use it.
    #[error("Error parsing: {0}")]
    ParserError(String),

    #[error("Hex decoding error: {0}")]
    HexDecodingError(#[from] FromHexError),
}
