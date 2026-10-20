use argh::FromArgs;
use minigit::MinigitError;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "log")]
/// Show commit logs
pub struct LogCommand {}

impl LogCommand {
    pub fn run(self) -> Result<(), MinigitError> {
        Ok(())
    }
}
