use std::{env, path::PathBuf, rc::Rc};

use argh::FromArgs;
use indicatif::ProgressBar;
use minigit::{MinigitError, Repo, storage::object::packed::Packfile};

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "unpack-objects")]
/// Unpact objects from a packed archive
pub struct UnpackObjectsCommand {
    #[argh(positional)]
    file: String, // TODO: should it support multiple ones?
}

impl UnpackObjectsCommand {
    pub fn run(self) -> Result<(), MinigitError> {
        let Self { file } = self;

        let repo = Repo::open(env::current_dir()?)?;

        let pack_path = PathBuf::from(file);

        let idx_path = pack_path.with_extension("idx");
        let idx_path = if idx_path.exists() {
            Some(idx_path)
        } else {
            None
        };

        let pack = Rc::new(Packfile::open(pack_path, idx_path)?);

        let bar = ProgressBar::new(pack.objects_count as u64);

        for lazy_obj in pack.lazy_objects() {
            let obj = lazy_obj.into_object()?;

            if let Err(err) = repo.store.write_loose_object(obj) {
                match err {
                    MinigitError::ObjectExists => {
                        // we ignore objects that already exist
                    }

                    other => return Err(other),
                }
            }
            bar.inc(1);
        }

        bar.finish();

        Ok(())
    }
}
