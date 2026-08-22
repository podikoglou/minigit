use std::{fs, io, path::PathBuf};

pub struct Store(PathBuf);

impl Store {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }

    /// Returns an iterator over pairs of object hashes and their paths.
    pub fn list_objects(&self) -> Result<impl Iterator<Item = (String, PathBuf)>, io::Error> {
        let objects_path = self.0.join("objects/");
        let entries = fs::read_dir(objects_path)?;

        Ok(entries.filter_map(Result::ok).flat_map(|prefix_dir| {
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
