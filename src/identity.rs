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
    derive(Debug, PartialEq, Eq, Clone, Display)
)]
pub struct Name(String);

#[nutype(
    sanitize(trim),
    validate(not_empty),
    derive(Debug, PartialEq, Eq, Clone, Display)
)]
pub struct Email(String);
