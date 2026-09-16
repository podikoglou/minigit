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
    pub extra: Vec<CommitProperty>,

    pub description: String,
}

impl Commit {
    pub fn new(
        tree: ObjectHash,
        parents: Vec<ObjectHash>,
        author: (Identity, DateTime<FixedOffset>),
        committer: (Identity, DateTime<FixedOffset>),
        extra: Vec<CommitProperty>,
        description: String,
    ) -> Self {
        Self {
            tree,
            parents,
            author,
            committer,
            extra,
            description,
        }
    }

    /// Attempts to get the GPG Signature (including the armor) used to sign this commit.
    pub fn gpg_signature(&self) -> Option<&String> {
        self.extra
            .iter()
            .find_map(|(key, value)| if key == "gpgsig" { Some(value) } else { None })
    }
}

impl WriteLoose for Commit {
    fn write_loose<W: Write>(&self, writer: &mut W) -> Result<(), crate::MinigitError> {
        writeln!(writer, "tree {}", self.tree)?;

        writeln!(writer, "author ")?;
        self.author.write_loose(writer)?;

        writeln!(writer, "committer ")?;
        self.committer.write_loose(writer)?;

        for property in &self.extra {
            property.write_loose(writer)?;
            writeln!(writer)?;
        }

        writeln!(writer)?;
        write!(writer, "{}", self.description)?;

        Ok(())
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
        write!(writer, "{} <{}>", self.name, self.email)?;

        Ok(())
    }
}

impl WriteLoose for (Identity, DateTime<FixedOffset>) {
    fn write_loose<W: Write>(&self, writer: &mut W) -> Result<(), crate::MinigitError> {
        self.0.write_loose(writer)?;
        write!(writer, " {}", self.1.format("%s %z"))?;

        Ok(())
    }
}

pub type CommitProperty = (String, String);

impl WriteLoose for &CommitProperty {
    fn write_loose<W: Write>(&self, writer: &mut W) -> Result<(), crate::MinigitError> {
        write!(writer, "{} {}", self.0, self.1)?;

        Ok(())
    }
}
