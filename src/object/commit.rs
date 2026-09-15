use crate::{object::hash::ObjectHash, storage::object::loose::WriteLoose};
use chrono::{DateTime, FixedOffset};
use std::io::Write;

/// A commit an object that contains information about a commit.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Commit {
    pub tree: ObjectHash,
    pub parents: Vec<ObjectHash>,
    pub author: (Identity, DateTime<FixedOffset>),
    pub committer: (Identity, DateTime<FixedOffset>),
    pub gpg_signature: Option<String>, // TODO: this belongs in `extra`
    pub extra: Vec<CommitProperty>,

    pub description: String,
}

impl Commit {
    pub fn new(
        tree: ObjectHash,
        parents: Vec<ObjectHash>,
        author: (Identity, DateTime<FixedOffset>),
        committer: (Identity, DateTime<FixedOffset>),
        gpg_signature: Option<String>,
        extra: Vec<CommitProperty>,
        description: String,
    ) -> Self {
        Self {
            tree,
            parents,
            author,
            committer,
            gpg_signature,
            extra,
            description,
        }
    }
}

impl WriteLoose for Commit {
    fn write_loose<W: Write>(&self, writer: &mut W) -> Result<(), crate::MinigitError> {
        todo!()
    }
}

/// Contains information about a person in Git (like an author or committer)
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Identity {
    pub name: String,
    pub email: String,
}

impl Identity {
    pub fn new(name: String, email: String) -> Self {
        Self { name, email }
    }
}

impl WriteLoose for Identity {
    fn write_loose<W: Write>(&self, writer: &mut W) -> Result<(), crate::MinigitError> {
        todo!()
    }
}

pub type CommitProperty = (String, String);
