//! This module deals with the modeling and valiation of user identities.
//!
//! It offers newtypes such as [`Identity`], which is consisted of a [`Name`] and [`Email`].

use nutype::nutype;

#[derive(Debug, PartialEq)]
pub struct Identity {
    pub name: Name,
    pub email: Email,
}

impl Identity {
    pub fn new(name: Name, email: Email) -> Self {
        Self { name, email }
    }
}

#[nutype(sanitize(trim), validate(not_empty), derive(Debug, PartialEq))]
pub struct Name(String);

#[nutype(sanitize(trim), validate(not_empty), derive(Debug, PartialEq))]
pub struct Email(String);
