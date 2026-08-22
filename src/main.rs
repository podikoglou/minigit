use std::{
    env,
    fs::{self},
    io::{self, Read},
};

use crate::{
    cli::{Options, options},
    object::{Object, ObjectType, blob::Blob},
    storage::Store,
};

pub mod hash;
pub mod object;
pub mod storage;

mod cli;

fn main() -> anyhow::Result<()> {
    let opts = options().run();

    let git_dir = env::current_dir()?.join(".git");
    let store = Store::new(git_dir);

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
        Options::LsObjectsCommand {} => {
            for object in store.objects()? {
                println!("{}", object.0);
            }
        }
    }

    Ok(())
}
