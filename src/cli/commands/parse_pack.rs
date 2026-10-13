use std::path::PathBuf;

use argh::FromArgs;
use minigit::{
    MinigitError,
    storage::object::packed::Packfile,
};

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "parse-pack")]
/// Reads a packfile from a file and parses it
pub struct ParsePackCommand {
    #[argh(positional)]
    file: String,
}

impl ParsePackCommand {
    pub fn run(self) -> Result<(), MinigitError> {
        let Self { file } = self;

        let pack_path = PathBuf::from(file);

        let idx_path = pack_path.with_extension("idx");
        let idx_path = if idx_path.exists() {
            Some(idx_path)
        } else {
            None
        };

        let pack = Packfile::open(pack_path, idx_path)?;

        for object in pack.objects() {
            println!("{object:#?}");
        }

        Ok(())
    }
}
