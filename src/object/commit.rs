use winnow::{
    ModalResult, Parser,
    combinator::{repeat, seq},
    error::StrContext,
    token::rest,
};

use crate::{
    identity::{Identity, parse_identity},
    object::hash::{ObjectHash, parse_object_hash_str},
    parsing::{Stream, extra_property, property},
    storage::object::loose::WriteLoose,
    time::{Timestamp, parse_timestamp},
};
use std::io::Write;

/// A commit an object that contains information about a commit.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Commit {
    pub tree: ObjectHash,
    pub parents: Vec<ObjectHash>,
    pub author: (Identity, Timestamp),
    pub committer: (Identity, Timestamp),
    pub extra: Vec<CommitProperty>,

    pub description: String,
}

impl Commit {
    pub fn new(
        tree: ObjectHash,
        parents: Vec<ObjectHash>,
        author: (Identity, Timestamp),
        committer: (Identity, Timestamp),
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

/// Parses a commit object from some bytes.
pub fn parse_commit<'a>(input: &mut Stream<'a>) -> ModalResult<Commit> {
    seq! {Commit{
        tree: property("tree", parse_object_hash_str),
        parents: repeat(0.., property("parent", parse_object_hash_str)),
        author: property("author", seq!(parse_identity, _: " ", parse_timestamp)),
        committer: property("committer", seq!(parse_identity, _: " ", parse_timestamp)),
        extra: repeat(0.., extra_property),
        _: "\n",
        description: rest.map(str::from_utf8).verify_map(Result::ok).map(str::to_owned),
    }}
    .context(StrContext::Label("commit object"))
    .parse_next(input)
}

impl WriteLoose for (Identity, Timestamp) {
    fn write_loose<W: Write>(&self, writer: &mut W) -> Result<(), crate::MinigitError> {
        self.0.write_loose(writer)?;
        write!(writer, " {}", self.1.format("%s %z"))?;

        Ok(())
    }
}

pub type CommitProperty = (String, String);

impl WriteLoose for &CommitProperty {
    fn write_loose<W: Write>(&self, writer: &mut W) -> Result<(), crate::MinigitError> {
        // TODO: validation. neither should be empty
        // the below code assumes they aren't

        // write key
        write!(writer, "{} ", self.0)?;

        let mut lines = self.1.lines();

        let Some(first_line) = lines.next() else {
            return Ok(()); // is this a failure?
        };

        // write first line of value
        write!(writer, "{first_line}")?;

        // write each subsequent line, indented by one space, indented by one space.
        //
        // (the reason we prefix with a newline rather than putting it at the end is
        // because we don't want to finish with one.)
        for line in lines {
            write!(writer, "\n {line}")?;
        }

        Ok(())
    }
}
