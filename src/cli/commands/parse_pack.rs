use std::fs::File;

use argh::FromArgs;
use memmap2::Mmap;
use minigit::{MinigitError, storage::object::packed};

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

        let file = File::open(file)?;

        let mmap = unsafe { Mmap::map(&file) }?;
        let mut slice = &mmap[..];

        packed::header(&mut slice).unwrap();

        loop {
            let object = match packed::object(&mut slice) {
                Ok(v) => v,
                Err(_) => break,
            };

            println!("{object:#?}");
        }

        Ok(())
    }
}
