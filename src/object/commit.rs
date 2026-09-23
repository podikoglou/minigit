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
    #[must_use]
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
    #[must_use]
    pub fn gpg_signature(&self) -> Option<&String> {
        self.extra
            .iter()
            .find_map(|(key, value)| if key == "gpgsig" { Some(value) } else { None })
    }
}

impl WriteLoose for Commit {
    fn write_loose<W: Write>(&self, writer: &mut W) -> Result<(), crate::MinigitError> {
        write!(writer, "tree {}", self.tree)?;
        writeln!(writer)?;

        for parent in &self.parents {
            write!(writer, "parent {}", parent)?;
            writeln!(writer)?;
        }

        write!(writer, "author ")?;
        self.author.write_loose(writer)?;
        writeln!(writer)?;

        write!(writer, "committer ")?;
        self.committer.write_loose(writer)?;
        writeln!(writer)?;

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
pub fn parse_commit(input: &mut Stream<'_>) -> ModalResult<Commit> {
    seq! {Commit{
        tree: property("tree", parse_object_hash_str),
        parents: repeat(0.., property("parent", parse_object_hash_str)),
        author: property("author", seq!(parse_identity, _: " ", parse_timestamp)),
        committer: property("committer", seq!(parse_identity, _: " ", parse_timestamp)),
        extra: repeat(0.., extra_property),
        _: "\n",
        description: rest.map(String::from_utf8_lossy).map(|str| str.to_string()),
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

#[cfg(test)]
mod tests {
    use chrono::{DateTime, FixedOffset};

    use crate::{
        identity::{Email, Identity, Name},
        object::{commit::Commit, hash::ObjectHash},
        storage::object::loose::WriteLoose,
        time::Timestamp,
    };

    #[test]
    fn writes_basic_commits() {
        let hash = ObjectHash::from([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let alex = (
            Identity::new(
                Name::try_new("alex".into()).unwrap(),
                Email::new("alex.podikoglou@gmail.com".into()),
            ),
            Timestamp::try_new(
                DateTime::from_timestamp(0, 0)
                    .unwrap()
                    .with_timezone(&FixedOffset::west_opt(0).unwrap()),
            )
            .unwrap(),
        );
        let commit = Commit::new(
            hash.clone(),
            vec![hash.clone(), hash],
            alex.clone(),
            alex,
            vec![],
            "test commit\nsome more text maybe\n".to_string(),
        );

        let mut buf = Vec::new();

        commit.write_loose(&mut buf).unwrap();

        assert_eq!(
            String::from_utf8(buf).unwrap(),
            r#"tree 0000000000000000000000000000000000000000
parent 0000000000000000000000000000000000000000
parent 0000000000000000000000000000000000000000
author alex <alex.podikoglou@gmail.com> 0 +0000
committer alex <alex.podikoglou@gmail.com> 0 +0000

test commit
some more text maybe
"#
        );
    }
}
