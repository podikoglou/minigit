use crate::object::hash::ObjectHash;
use chrono::{DateTime, FixedOffset};
use std::io::Write;

/// A commit an object that contains information about a commit.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Commit {
    pub tree: ObjectHash,
    pub author: (Identity, DateTime<FixedOffset>),
    pub committer: (Identity, DateTime<FixedOffset>),
}

impl Commit {
    pub fn new(
        tree: ObjectHash,
        author: (Identity, DateTime<FixedOffset>),
        committer: (Identity, DateTime<FixedOffset>),
    ) -> Self {
        Self {
            tree,
            author,
            committer,
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
