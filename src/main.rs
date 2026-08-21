use std::{
    fs::{self, File},
    io::{self, Read},
};

use crate::{
    cli::{Options, options},
    object::ObjectType,
};

pub mod hash;
pub mod object;

mod cli;

fn main() -> anyhow::Result<()> {
    let opts = options().run();

    match opts {
        Options::HashObjectCommand {
            r#type,
            stdin,
            write,
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

                        loop {
                            match lock.read(&mut buf) {
                                Ok(v) if v != 0 => continue,
                                _ => break,
                            }
                        }

                        Some(buf)
                    } else {
                        None
                    };

                    let files = stdin_data
                        .map(Ok)
                        .into_iter()
                        .chain(files.into_iter().map(fs::read));

                    for file in files {
                        let file = file?;

                        dbg!(file);
                    }
                }
                ObjectType::Tree => {
                    todo!()
                }
            }
        }
    }

    Ok(())
}
