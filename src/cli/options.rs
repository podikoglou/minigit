use argh::FromArgs;
use minigit::MinigitError;

use crate::cli::commands::{
    hash_object::HashObjectCommand, ls_buckets::LsBucketsCommand, ls_objects::LsObjectsCommand,
    parse_object::ParseObjectCommand,
};

/// The stupid implementation of the stupid content tracker.
#[derive(FromArgs, PartialEq, Debug)]
pub struct Args {
    #[argh(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum Subcommand {
    HashObject(HashObjectCommand),
    LsBuckets(LsBucketsCommand),
    LsObjects(LsObjectsCommand),
    ParseObject(ParseObjectCommand),
}

impl Args {
    pub fn run(self) -> Result<(), MinigitError> {
        match self.subcommand {
            Subcommand::HashObject(cmd) => cmd.run(),
            Subcommand::LsBuckets(cmd) => cmd.run(),
            Subcommand::LsObjects(cmd) => cmd.run(),
            Subcommand::ParseObject(cmd) => cmd.run(),
        }
    }
}
