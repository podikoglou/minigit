//! This module deals with the modeling and valiation of user identities.
//!
//! It offers newtypes such as [`Identity`], which is consisted of a [`Name`] and [`Email`].

use std::io::Write;

use nutype::nutype;

use crate::storage::object::loose::WriteLoose;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Identity {
    pub name: Name,
    pub email: Email,
}

impl Identity {
    pub fn new(name: Name, email: Email) -> Self {
        Self { name, email }
    }
}

impl WriteLoose for Identity {
    fn write_loose<W: Write>(&self, writer: &mut W) -> Result<(), crate::MinigitError> {
        write!(writer, "{} <{}>", self.name, self.email)?;

        Ok(())
    }
}

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, PartialEq, Eq, Clone, Display, AsRef, Deref)
)]
pub struct Name(String);

impl PartialEq<&str> for Name {
    fn eq(&self, other: &&str) -> bool {
        self.as_ref() == *other
    }
}

impl PartialEq<str> for Name {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl PartialEq<Name> for &str {
    fn eq(&self, other: &Name) -> bool {
        *self == other.as_ref()
    }
}

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, PartialEq, Eq, Clone, Display, AsRef, Deref)
)]
pub struct Email(String);

impl PartialEq<&str> for Email {
    fn eq(&self, other: &&str) -> bool {
        self.as_ref() == *other
    }
}

impl PartialEq<str> for Email {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl PartialEq<Email> for &str {
    fn eq(&self, other: &Email) -> bool {
        *self == other.as_ref()
    }
}

