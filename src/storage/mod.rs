use std::{
    fs::{self, DirEntry},
    io,
    path::PathBuf,
};

use anyhow::bail;

pub mod object;
pub mod prefix;

pub struct Store {
    path: PathBuf,
}

impl Store {
    pub fn try_new(path: PathBuf) -> Result<Self, anyhow::Error> {
        match fs::exists(&path) {
            Ok(true) => Ok(Self { path }),
            Ok(false) => bail!("directory does not exist"),
            Err(err) => bail!(err),
        }
    }

    pub fn objects_path(&self) -> PathBuf {
        self.0.join("objects/")
    }

    /// Returns an iterator over the paths of the prefix directories.
    pub fn prefix_dirs(&self) -> Result<impl Iterator<Item = DirEntry>, io::Error> {
        let entries = fs::read_dir(self.objects_path())?;

        Ok(entries.filter_map(Result::ok))
    }

    /// Returns an iterator over pairs of object hashes and their paths.
    pub fn objects(&self) -> Result<impl Iterator<Item = (String, PathBuf)>, io::Error> {
        let prefix_dirs = self.prefix_dirs()?;

        Ok(prefix_dirs
            .filter(|entry| entry.file_name().len() == 2)
            .flat_map(|prefix_dir| {
                let prefix = prefix_dir.file_name();

                fs::read_dir(prefix_dir.path())
                    .into_iter()
                    .flatten()
                    .filter_map(Result::ok)
                    .map(move |file| {
                        (
                            format!(
                                "{}{}",
                                prefix.to_string_lossy(),
                                file.file_name().to_string_lossy()
                            ),
                            file.path(),
                        )
                    })
            }))
    }
}
