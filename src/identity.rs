//! This module deals with the modeling and valiation of user identities.
//!
//! It offers newtypes such as [`Identity`], which is consisted of a [`Name`] and [`Email`].

use nutype::nutype;

pub type Identity = (Name, Email);

#[nutype(sanitize(trim), validate(not_empty), derive(Debug, PartialEq))]
pub struct Name(String);

#[nutype(sanitize(trim), validate(not_empty), derive(Debug, PartialEq))]
pub struct Email(String);
