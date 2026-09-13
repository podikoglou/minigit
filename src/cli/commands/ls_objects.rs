use std::env;

use bpaf::Bpaf;
use minigit::{
    MinigitError, Repo,
    object::hash::HashPrefix,
    storage::object::{LazyObject, ObjectsBucket},
};
use strum::IntoDiscriminant;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(command("ls-objects"))]
/// List objects in the repository
pub struct LsObjectsCommand {
    buckets: Vec<HashPrefix>,
    pretty: bool,
}

impl LsObjectsCommand {
    pub fn run(self) -> Result<(), MinigitError> {
        let LsObjectsCommand { buckets, pretty } = self;
        let repo = Repo::open(env::current_dir()?)?;

        // TODO: can we remove this box?
        let objects: Box<dyn Iterator<Item = Result<LazyObject, MinigitError>>> =
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

        if !pretty {
            // default output, just print object hashes
            for object in objects {
                println!("{}", object?.hash);
            }
        } else {
            // pretty output, print object types
            for lazy_object in objects.filter_map(Result::ok) {
                let object = lazy_object.into_object()?;

                println!("{} {:?}", lazy_object.hash, object.discriminant());
            }
        }

        Ok(())
    }
}
