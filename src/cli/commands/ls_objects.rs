use std::env;

use bpaf::Bpaf;
use minigit::{
    Repo,
    object::hash::HashPrefix,
    storage::object::{LazyObject, ObjectsBucket},
};

#[derive(Debug, Clone, Bpaf)]
#[bpaf(command("ls-objects"))]
/// List objects in the repository
pub struct LsObjectsCommand {
    buckets: Vec<HashPrefix>,
}

impl LsObjectsCommand {
    pub fn run(self) -> Result<(), anyhow::Error> {
        let LsObjectsCommand { buckets } = self;
        let repo = Repo::open(env::current_dir()?)?;

        // TODO: can we remove this box?
        let objects: Box<dyn Iterator<Item = Result<LazyObject, anyhow::Error>>> =
            if buckets.is_empty() {
                Box::new(repo.store.objects()?)
            } else {
                Box::new(
                    buckets
                        .into_iter()
                        .filter_map(|prefix| repo.store.bucket(prefix).ok())
                        .map(ObjectsBucket::objects)
                        .filter_map(Result::ok)
                        .flatten(),
                )
            };

        for object in objects {
            println!("{}", object?.hash);
        }

        Ok(())
    }
}
