use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MinigitError {
    #[error("IO Error")]
    IO(#[from] io::Error),

    #[error("Invalid file name")]
    InvalidFileName,

    #[error("No git directory")]
    NoGitDirectory,

    #[error("Bucket not found")]
    BucketNotFound,

    #[error("Object not found")]
    ObjectNotFound,
}
