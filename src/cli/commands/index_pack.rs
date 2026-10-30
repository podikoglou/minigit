use std::path::PathBuf;

use argh::FromArgs;
use tinygit::{
    TinygitError,
    storage::object::packed::idx_builder::{ObjectCache, PackIndexBuilder},
};

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "index-pack")]
/// Build pack index file for an existing packed archive
pub struct IndexPackCommand {
    #[argh(positional)]
    file: PathBuf,
}

impl IndexPackCommand {
    pub fn run(self) -> Result<(), TinygitError> {
        let Self { file } = self;

        let builder = PackIndexBuilder::new(file, None)?;
        let mut cache = ObjectCache::default();

        let index = builder.build(&mut cache)?;

        index.write()?;

        Ok(())
    }
}
