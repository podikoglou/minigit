use std::{
    fs::{self, File},
    io::BufWriter,
    path::PathBuf,
    rc::Rc,
};

use itertools::Itertools;
use walkdir::WalkDir;

use crate::{
    MinigitError,
    error::ParserContext,
    object::{
        Object,
        hash::{HashPrefix, ObjectHash},
    },
    storage::{
        object::{LazyObject, ObjectsBucket, RawObject, packed::Packfile},
        reference::{Ref, RefName, RefPointer, db::RefsDb},
    },
};

pub mod object;
pub mod reference;

#[derive(Debug, Default)]
pub struct Store {
    pub path: PathBuf,

    packs: Vec<Rc<Packfile>>,
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

    #[must_use]
    pub fn refs_path(&self) -> PathBuf {
        self.path.join("refs/")
    }

    #[must_use]
    pub fn ref_path(&self, name: &String) -> PathBuf {
        self.path.join(name)
    }

    #[must_use]
    pub fn packed_refs_path(&self) -> PathBuf {
        self.path.join("packed-refs")
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

                    Some(Rc::new(pack))
                })
                .collect::<Vec<_>>()
        } else {
            Vec::default()
        };

        Ok(())
    }

    /// Gets all the [Packfile]s in the store.
    pub fn packs(&self) -> impl Iterator<Item = Rc<Packfile>> {
        self.packs.iter().cloned()
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

    /// Opens the `packed-refs` database
    pub fn ref_db(&self) -> Result<Rc<RefsDb>, MinigitError> {
        // TODO: open it once!
        RefsDb::open(self.packed_refs_path()).map(Rc::new)
    }

    /// Finds a [Ref] in the repository.
    ///
    /// Note that this function does not guarantee that the returned ref is peeled.
    pub fn reference(&self, name: RefName) -> Result<Ref, MinigitError> {
        // try to read as loose
        //
        // NOTE: this will be unpeeled!
        let path = self.ref_path(&name);
        if path.exists() {
            return Ref::open_loose(path);
        }

        // try to read packed
        // TODO: Is this good error handling? I don't like how we completely ignore the underlying
        // error, but the truth is that, yes, indeed, the ref was not found, no matter why. Perhaps
        // RefNotFound could contain Option<MinigitError> ?

        let ref_db = self.ref_db().map_err(|_| MinigitError::RefNotFound)?;

        ref_db.lookup(&name).map_err(|_| MinigitError::RefNotFound)
    }

    // /// Returns an iterator over the [UnresolvedRef]s in the repository.
    // pub fn unresolved_refs(&self) -> impl Iterator<Item = Result<UnresolvedRef, MinigitError>> {
    //     WalkDir::new(self.refs_path())
    //         .into_iter()
    //         .filter_map(|entry| match entry {
    //             Ok(entry) if entry.file_type().is_file() => Some(UnresolvedRef::open(entry.path())),
    //
    //             Ok(_) => None,
    //
    //             Err(err) => Some(Err(MinigitError::IO(err.into()))),
    //         })
    // }
    //
    // /// Resolves a [RefPointer] into an [ObjectHash].
    // pub fn resolve_ref_pointer(&self, pointer: RefPointer) -> Result<ObjectHash, MinigitError> {
    //     match pointer {
    //         RefPointer::Object(hash) => Ok(hash),
    //         RefPointer::Ref(path) => self.resolve_ref_pointer(UnresolvedRef::open(path)?.pointer),
    //     }
    // }
}
