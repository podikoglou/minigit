//! This module deals with the storage and parsing of refs (`.git/refs`)

use std::{path::PathBuf, rc::Rc};

use crate::object::hash::ObjectHash;

/// A resolved ref pointing to an object.
#[derive(Debug, PartialEq, Eq)]
pub struct Ref {
    pub path: PathBuf,
    pub pointer: ObjectHash,
}

/// An unresolved ref pointing to an object.
///
/// There are two possible cases for lazy refs:
/// - the `pointer` is already a hash (but still requires resolving to be turned to a [`Ref`])
/// - the `pointer` is the name of another ref, which must be resolved
///
/// These are resolved through [`crate::storage::Store`]
#[derive(Debug, PartialEq, Eq)]
pub struct LazyRef {
    pub path: PathBuf,
    pub pointer: RefPointer,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RefPointer {
    Object(ObjectHash),
    Ref(PathBuf),
}
