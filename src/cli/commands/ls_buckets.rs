use std::env;

use argh::FromArgs;
use tinygit::{TinygitError, Repo};

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "ls-buckets")]
/// List buckets in the repository
pub struct LsBucketsCommand {}

impl LsBucketsCommand {
    pub fn run(self) -> Result<(), TinygitError> {
        let repo = Repo::open(env::current_dir()?)?;

        for bucket in repo.store.buckets()? {
            println!("{}", bucket.prefix);
        }

        Ok(())
    }
}
