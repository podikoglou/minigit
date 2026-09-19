use std::{
    env,
    io::{self, BufWriter, Write},
};

use argh::FromArgs;
use minigit::{
    MinigitError, Repo,
    object::hash::HashPrefix,
    storage::object::{LazyObject, ObjectsBucket},
};
use strum::IntoDiscriminant;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "ls-objects")]
/// List objects in the repository
pub struct LsObjectsCommand {
    /// the buckets to read objets from.
    #[argh(positional)]
    buckets: Vec<HashPrefix>,

    /// whether to make the output pretty and include more information
    #[argh(switch)]
    pretty: bool,

    /// whether to make the output pretty and include more information (second version)
    #[argh(switch)]
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

        let stdout = io::stdout();
        let mut out = BufWriter::new(stdout.lock());

        match (pretty, pretty_2) {
            (false, false) => {
                // default output, just print object hashes
                for object in objects {
                    writeln!(out, "{}", object?.hash()?)?;
                }
            }
            (true, _) => {
                for lazy_object in objects.filter_map(Result::ok) {
                    let object = lazy_object.into_object()?;

                    writeln!(out, "{} {:?}", lazy_object.hash()?, object.discriminant())?;
                }
            }
            (_, true) => {
                // pretty output, print object types
                for lazy_object in objects.filter_map(Result::ok) {
                    let object = lazy_object.into_object()?;
                    let hash = lazy_object.hash()?;

                    match object {
                        minigit::object::Object::Blob(blob) => {
                            writeln!(out, "{}  blob    {} bytes", hash, blob.0.len())
                        }
                        minigit::object::Object::Tree(tree) => {
                            writeln!(out, "{}  tree    {} entries", hash, tree.entries.len())
                        }
                        minigit::object::Object::Commit(commit) => writeln!(
                            out,
                            "{}  commit  {}",
                            hash,
                            commit.description.lines().next().unwrap_or_default(),
                        ),
                        minigit::object::Object::Tag(tag) => {
                            writeln!(out, "{}  tag  {}", hash, tag.name)
                        }
                    }?;
                }
            }
        };

        Ok(())
    }
}
