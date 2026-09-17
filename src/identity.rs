//! This module deals with the modeling and valiation of user identities.
//!
//! It offers newtypes such as [`Identity`], which is consisted of a [`Name`] and [`Email`].

use std::io::Write;

use nutype::nutype;
use winnow::{
    ModalResult, Parser,
    combinator::seq,
    error::{StrContext, StrContextValue},
    token::take_until,
};

use crate::{parsing::Stream, storage::object::loose::WriteLoose};

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
    validate(len_char_min = 0),
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

/// Parses an identity in the form of `John Doe <john@doe.com>` from some bytes.
pub fn parse_identity<'a>(input: &mut Stream<'a>) -> ModalResult<Identity> {
    seq!(take_until(0.., " <").map(String::from_utf8_lossy).map(Name::try_new).verify_map(Result::ok).context(StrContext::Label("name")),
        _: " <",
        take_until(0.., ">").map(String::from_utf8_lossy).map(Email::new).context(StrContext::Label("email")),
        _: ">"
    )
    .map(|(name, email)| Identity::new(name, email))
    .context(StrContext::Label("identity"))
    .context(StrContext::Expected(StrContextValue::Description(
        "<name> <<email>>",
    )))
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use winnow::Parser;

    use crate::identity::{Email, Identity, Name, parse_identity};
    use std::assert_matches;

    #[test]
    fn identity_parses_valid_identities() {
        assert_eq!(
            parse_identity.parse_peek(b"John Doe <john@doe.com>"),
            Ok((
                &b""[..],
                Identity::new(
                    Name::try_new("John Doe").unwrap(),
                    Email::new("john@doe.com")
                )
            ))
        );
    }

    #[test]
    fn identity_rejects_invalid_input() {
        assert_matches!(parse_identity.parse_peek(b"<john@doe.com>"), Err(_));
        assert_matches!(parse_identity.parse_peek(b"j<john@doe.com>"), Err(_));
        assert_matches!(parse_identity.parse_peek(b"<john@doe.com"), Err(_));
        assert_matches!(parse_identity.parse_peek(b"john@doe.com>"), Err(_));
        assert_matches!(parse_identity.parse_peek(b"john@doe.com"), Err(_));
        // TODO: should this validate emails?
        assert_matches!(parse_identity.parse_peek(b"johndoe.com"), Err(_));
    }
}
