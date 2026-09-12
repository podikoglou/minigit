use std::env;

use bpaf::Bpaf;
use minigit::Repo;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(command("ls-buckets"))]
/// List buckets in the repository
pub struct LsBucketsCommand {}

impl LsBucketsCommand {
    pub fn run(self) -> Result<(), anyhow::Error> {
    let repo = Repo::open(env::current_dir()?)?;

    for bucket in repo.store.buckets()? {
        println!("{}", bucket.prefix);
    }

    Ok(())
    }
}
