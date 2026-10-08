use std::fs::File;

use argh::FromArgs;
use memmap2::Mmap;
use minigit::{MinigitError, parsing::Stream, storage::object::packed};

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
        let slice = &mmap[..];
        let mut stream = Stream::new(slice);

        packed::header(&mut stream).unwrap();

        loop {
            let object = match packed::object(&mut stream) {
                Ok(v) => v,
                Err(_) => break,
            };

            println!("{object:#?}");
        }

        Ok(())
    }
}
