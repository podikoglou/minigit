use bpaf::Bpaf;
use minigit::MinigitError;

use crate::cli::commands::{
    hash_object::{HashObjectCommand, hash_object_command},
    ls_buckets::{LsBucketsCommand, ls_buckets_command},
    ls_objects::{LsObjectsCommand, ls_objects_command},
};

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub enum Options {
    HashObject(#[bpaf(external(hash_object_command))] HashObjectCommand),
    LsBuckets(#[bpaf(external(ls_buckets_command))] LsBucketsCommand),
    LsObjects(#[bpaf(external(ls_objects_command))] LsObjectsCommand),
}

impl Options {
    pub fn run(self) -> Result<(), MinigitError> {
        match self {
            Self::HashObject(cmd) => cmd.run(),
            Self::LsBuckets(cmd) => cmd.run(),
            Self::LsObjects(cmd) => cmd.run(),
        }
    }
}
