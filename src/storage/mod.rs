use std::{fs, io, path::PathBuf};

pub struct Store(PathBuf);

impl Store {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }

    pub fn objects_path(&self) -> PathBuf {
        self.0.join("objects/")
    }

    /// Returns an iterator over pairs of object hashes and their paths.
    pub fn objects(&self) -> Result<impl Iterator<Item = (String, PathBuf)>, io::Error> {
        let entries = fs::read_dir(self.objects_path())?;

        Ok(entries
            .filter_map(Result::ok)
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
