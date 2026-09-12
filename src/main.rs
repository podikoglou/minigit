use crate::cli::options;

mod cli;

fn main() -> anyhow::Result<()> {
    options().run().run()
}
