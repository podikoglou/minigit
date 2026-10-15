//! This module deals with the storage and parsing of refs (`.git/refs`)

use std::{path::PathBuf, rc::Rc};

use crate::object::hash::ObjectHash;

#[derive(Debug, PartialEq, Eq)]
pub struct Ref {
    pub path: PathBuf,
    pub pointer: RefPointer,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RefPointer {
    Object(ObjectHash),
    Ref(Rc<Ref>),
}
