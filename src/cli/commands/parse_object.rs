use std::{
    fs::{self},
    io::{self, Read},
    path::PathBuf,
};

use argh::FromArgs;
use minigit::error::ParserContext;
use minigit::{MinigitError, storage::object::loose::read_object_compressed};

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "parse-object")]
/// Reads an object from a file or stdin and parses it
pub struct ParseObjectCommand {
    /// read the object from the standard input
    #[argh(switch)]
    stdin: bool,

    #[argh(positional)]
    files: Vec<String>,
}

impl ParseObjectCommand {
    pub fn run(self) -> Result<(), MinigitError> {
        let Self { stdin, files } = self;

        // if stdin, deal with this first
        let stdin_data: Option<Vec<u8>> = if stdin {
            let stdin = io::stdin();
            let mut buf = Vec::new();

            let mut lock = stdin.lock();

            lock.read_to_end(&mut buf)?;

            Some(buf)
        } else {
            None
        };

        let objects = stdin_data
            .map(|data| (data, ParserContext::None))
            .map(Ok)
            .into_iter()
            .chain(
                files
                    .into_iter()
                    .map(PathBuf::from)
                    .map(|path| fs::read(&path).map(|bytes| (bytes, ParserContext::File(path)))),
            )
            .map(|contents| match contents {
                Ok((bytes, context)) => read_object_compressed(bytes.as_slice(), context),
                Err(err) => Err(MinigitError::from(err)),
            });

        for object in objects {
            match object {
                Ok(object) => {
                    println!("{object:#?}");
                }
                Err(err) => eprintln!("{err}"),
            }
        }

        Ok(())
    }
}
