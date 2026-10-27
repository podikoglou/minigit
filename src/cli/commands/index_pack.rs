use argh::FromArgs;
use minigit::MinigitError;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "index-pack")]
/// Build pack index file for an existing packed archive
pub struct IndexPackCommand {
    #[argh(positional)]
    file: String,
}

impl IndexPackCommand {
    pub fn run(self) -> Result<(), MinigitError> {
        let Self { file } = self;

        Ok(())
    }
}
