use std::{
    fs,
    io::{self, Read},
};

use bpaf::Bpaf;
use minigit::{
    MinigitError,
    object::{Object, ObjectType, blob::Blob},
};

#[derive(Debug, Clone, Bpaf)]
#[bpaf(command("hash-object"))]
/// Compute object ID and optionally create an object from a file
pub struct HashObjectCommand {
    /// Specify the type of the object to be created (default: "blob").
    /// Possible values are blob, tree.
    #[bpaf(long("type"), short('t'))]
    r#type: Option<ObjectType>,

    /// Read the object from the standard input instead of from a file.
    #[bpaf(flag(true, false))]
    stdin: bool,

    /// Actually write the object into the object database.
    #[bpaf(short('w'))]
    write: bool,

    #[bpaf(positional("file"))]
    files: Vec<String>,
}

impl HashObjectCommand {
    pub fn run(self) -> Result<(), MinigitError> {
        let HashObjectCommand {
            r#type,
            stdin,
            write: _,
            files,
        } = self;

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

                            Ok(Object::Blob(blob))
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
            ObjectType::Commit => {
                todo!()
            }
        }

        Ok(())
    }
}
