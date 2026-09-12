use minigit::MinigitError;

use crate::cli::options;

mod cli;

fn main() -> Result<(), MinigitError> {
    options().run().run()
}
