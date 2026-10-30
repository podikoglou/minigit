use std::path::PathBuf;

use argh::FromArgs;
use fallible_iterator::FallibleIterator;
use tinygit::{TinygitError, storage::object::packed::Packfile};

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "parse-pack")]
/// Reads a packfile from a file and parses it
pub struct ParsePackCommand {
    #[argh(positional)]
    file: String,
}

impl ParsePackCommand {
    pub fn run(self) -> Result<(), TinygitError> {
        let Self { file } = self;

        let pack_path = PathBuf::from(file);
        let idx_path = pack_path.with_extension("idx");

        let pack = Packfile::open(pack_path, idx_path)?;

        for object in pack.objects().iterator() {
            let object = object?;

            println!("{object:#?}");
        }

        Ok(())
    }
}
