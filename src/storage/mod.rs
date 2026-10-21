use std::{
    collections::HashMap,
    fs::{self, File},
    io::BufWriter,
    path::PathBuf,
    rc::Rc,
};

use itertools::Itertools;

use crate::{
    MinigitError,
    error::ParserContext,
    object::{
        Object,
        hash::{HashPrefix, ObjectHash},
    },
    storage::object::{LazyObject, ObjectsBucket, RawObject, packed::Packfile},
};

pub mod object;

#[derive(Debug, Default)]
pub struct Store {
    pub path: PathBuf,

    /// Mapping from pack name (such as 9b6ed366de8f29d9ebc6e1d44188de219a5ac3f3) to a [`Rc<Pack>`]
    packs: HashMap<String, Rc<Packfile>>,
}

impl Store {
    /// Opens an already existing git database.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, MinigitError> {
        let path = path.into();

        match fs::exists(&path) {
            Ok(true) => {
                let mut store = Self {
                    path,
                    ..Default::default()
                };

                store.refresh_packs()?;

                Ok(store)
            }
            Ok(false) => Err(MinigitError::NoGitDirectory),
            Err(err) => Err(err.into()),
        }
    }

    #[must_use]
    pub fn objects_path(&self) -> PathBuf {
        self.path.join("objects/")
    }

    #[must_use]
    pub fn packs_path(&self) -> PathBuf {
        self.objects_path().join("pack/")
    }

    /// Returns all buckets (directories named after the prefix of a hash) under `.git/objects/`.
    ///
    /// At most 256 of them exist, so they are collected eagerly.
    pub fn buckets(&self) -> Result<Vec<ObjectsBucket>, MinigitError> {
        fs::read_dir(self.objects_path())?
            .filter_map(|result| match result {
                Err(err) => Some(Err(err.into())),
                Ok(entry) if entry.file_name().len() == 2 => {
                    Some(ObjectsBucket::open(entry.path()))
                }
                Ok(_) => None,
            })
            .collect()
    }

    /// Finds an [`ObjectsBucket`] in .git/objects/
    pub fn bucket(&self, prefix: HashPrefix) -> Result<ObjectsBucket, MinigitError> {
        self.buckets()?
            .into_iter()
            .find(|dir| dir.prefix == prefix)
            .ok_or(MinigitError::BucketNotFound)
    }

    /// Refreshes the internal packfile listing
    pub fn refresh_packs(&mut self) -> Result<(), MinigitError> {
        let pack_dir = self.packs_path();

        self.packs = if pack_dir.exists() {
            fs::read_dir(pack_dir)?
                .filter_map(|entry| {
                    // it should be noted that this closure doesn't propagate any errors up; when an
                    // error is encountered, it is simply ignored
                    let path = entry.ok()?.path();
                    let file_name = path.file_name()?.to_string_lossy();

                    // ensure in the form of "pack-<hash>.pack"
                    if !file_name.starts_with("pack-") {
                        return None;
                    }

                    if !file_name.ends_with(".pack") {
                        return None;
                    }

                    let idx_path = path.with_extension("idx");

                    // ensure respective .idx existts
                    if !idx_path.exists() {
                        return None;
                    }

                    // TODO: when do we ever handle such failures? this seems important
                    let pack = Packfile::open(path, idx_path).ok()?;

                    Some((pack.name.clone(), Rc::new(pack)))
                })
                .collect::<HashMap<_, _>>()
        } else {
            HashMap::new()
        };

        Ok(())
    }

    /// Gets all the [Packfile]s in the store.
    pub fn packs(&self) -> impl Iterator<Item = Rc<Packfile>> {
        self.packs.values().map(|x| x.clone())
    }

    pub fn objects(
        &self,
    ) -> Result<impl Iterator<Item = Result<LazyObject, MinigitError>>, MinigitError> {
        let packs = self.packs();

        let objects = packs.into_iter().flat_map(Packfile::lazy_objects).map(Ok);

        Ok(self
            .buckets()?
            .into_iter()
            .map(ObjectsBucket::objects)
            .flatten_ok()
            .map(Result::flatten)
            .chain(objects))
    }

    /// Gets a [RawObject] by its [ObjectHash].
    pub fn raw_object(&self, hash: ObjectHash) -> Result<RawObject, MinigitError> {
        // search in packfiles
        for pack in self.packs() {
            if let Ok(packed) = pack.read_object_by_id(hash.clone()) {
                return pack.resolve(packed);
            }
        }

        // search in loose objects
        let path = self
            .objects_path()
            .join(hash.prefix().to_string())
            .join(&hash.to_string()[2..]); // <-- TODO: this could be a function of `ObjectHash`

        LazyObject::loose(path).into_raw()
    }

    /// Gets a [LazyObject] by its [ObjectHash].
    pub fn object(&self, hash: ObjectHash) -> Result<Object, MinigitError> {
        // search in packfiles
        for pack in self.packs() {
            if let Ok(packed) = pack.read_object_by_id(hash.clone()) {
                return pack
                    .resolve(packed)
                    .and_then(|obj| obj.into_object(ParserContext::File(pack.path().to_owned())));
            }
        }

        // search in loose objects
        let path = self
            .objects_path()
            .join(hash.prefix().to_string())
            .join(&hash.to_string()[2..]); // <-- TODO: this could be a function of `ObjectHash`

        LazyObject::loose(path).into_object()
    }

    /// Writes a [RawObject] to the loose object database.
    pub fn write_loose_object(&self, object: RawObject) -> Result<(), MinigitError> {
        let hash = object.hash()?;
        let prefix = hash.prefix();

        let bucket_path = self.objects_path().join(prefix.to_string());

        if !bucket_path.exists() {
            fs::create_dir_all(&bucket_path)?;
        }

        let object_path = bucket_path.join(&hash.to_string()[2..]);

        let object_file = File::create_new(object_path).map_err(|_| MinigitError::ObjectExists)?;
        let mut writer = BufWriter::new(object_file);

        object.write_loose_compressed(&mut writer)
    }
}
