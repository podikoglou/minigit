use std::{
    env,
    fs::{self},
    io::{self, Read},
};

use crate::{
    cli::{Options, options},
    object::{Object, ObjectType, blob::Blob},
    storage::{Store, object::LazyObject, prefix_dir::PrefixDir},
};

pub mod object;
pub mod storage;

mod cli;

fn main() -> anyhow::Result<()> {
    let opts = options().run();

    let git_dir = env::current_dir()?.join(".git");
    let store = Store::try_new(git_dir)?;

    match opts {
        Options::HashObjectCommand {
            r#type,
            stdin,
            write: _write,
            files,
        } => {
            let ty = r#type.unwrap_or(ObjectType::Blob);

            match ty {
                ObjectType::Blob => {
                    // if stdin, deal with this first
                    let stdin_data: Option<Vec<u8>> = if stdin {
                        let stdin = io::stdin();
                        let mut buf = Vec::new();

                        let mut lock = stdin.lock();

                        lock.read_to_end(&mut buf)?;

                        Some(buf)
                    } else {
                        None
                    };

                    let objects = stdin_data
                        .map(Ok)
                        .into_iter()
                        .chain(files.into_iter().map(fs::read))
                        .map(|contents| match contents {
                            Ok(bytes) => {
                                // create a Blob out of this
                                let blob = Blob::new(bytes);

                                Ok(Object::blob(blob))
                            }
                            Err(err) => Err(err),
                        });

                    for object in objects {
                        match object {
                            Ok(object) => println!("{}", object.hash()?),
                            Err(err) => println!("{}", err),
                        }
                    }
                }
                ObjectType::Tree => {
                    todo!()
                }
            }
        }
        Options::LsObjectsCommand { prefix_dirs } => {
            // TODO: can we remove this box?
            let objects: Box<dyn Iterator<Item = Result<LazyObject, anyhow::Error>>> =
                if prefix_dirs.is_empty() {
                    Box::new(store.objects()?)
                } else {
                    Box::new(
                        prefix_dirs
                            .into_iter()
                            .filter_map(|prefix| store.prefix_dir(prefix).ok())
                            .map(PrefixDir::objects)
                            .filter_map(Result::ok)
                            .flatten(),
                    )
                };

            for object in objects {
                println!("{}", object?.hash);
            }
        }
        Options::LsPrefixDirs {} => {
            for prefix_dir in store.prefix_dirs()? {
                println!("{}", prefix_dir.prefix);
            }
        }
    }

    Ok(())
}
