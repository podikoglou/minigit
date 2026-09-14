use crate::object::hash::ObjectHash;
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

impl Commit {
    /// Writes the commit into a writer.
    pub fn write<W: Write>(&self, mut writer: W) -> Result<(), std::io::Error> {
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

impl Identity {
    /// Writes the identity into a writer.
    pub fn write<W: Write>(&self, mut writer: W) -> Result<(), std::io::Error> {
        todo!()
    }
}

pub type CommitProperty = (String, String);
