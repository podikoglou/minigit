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
    pretty_2: bool,
}

impl LsObjectsCommand {
    pub fn run(self) -> Result<(), MinigitError> {
        let LsObjectsCommand {
            buckets,
            pretty,
            pretty_2,
        } = self;
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

        match (pretty, pretty_2) {
            (false, false) => {
                // default output, just print object hashes
                for object in objects {
                    println!("{}", object?.hash()?);
                }
            }
            (true, _) => {
                for lazy_object in objects.filter_map(Result::ok) {
                    let object = lazy_object.into_object()?;

                    println!("{} {:?}", lazy_object.hash()?, object.discriminant());
                }
            }
            (_, true) => {
                // pretty output, print object types
                for lazy_object in objects.filter_map(Result::ok) {
                    let object = lazy_object.into_object()?;
                    let hash = lazy_object.hash()?;

                    match object {
                        minigit::object::Object::Blob(blob) => {
                            println!("{}  blob    {} bytes", hash, blob.0.len())
                        }
                        minigit::object::Object::Tree(tree) => {
                            println!("{}  tree    {} entries", hash, tree.entries.len())
                        }
                        minigit::object::Object::Commit(commit) => {
                            println!(
                                "{}  commit  {}",
                                hash,
                                commit.description.lines().next().unwrap_or_default()
                            )
                        }
                    }
                }
            }
        };

        Ok(())
    }
}
