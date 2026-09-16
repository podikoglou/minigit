//! This module contains items that deal with file names.

use crate::{MinigitError, error::ParserContext, storage::object::loose::WriteLoose};
use std::{fmt::Display, path::PathBuf, str::FromStr};
use winnow::{ModalResult, Parser, combinator::terminated, token::take_until};

/// A file name.
///
/// This is a newtype which is validated upon creation, thus a [`FileName`] is assumed to be a valid
/// file name that Git supports.
///
/// A [`FileName`] can be created using [`FromStr`]. For example:
/// ```
/// use minigit::fs::FileName;
///
/// let name: FileName = "foo.rs\0".parse().unwrap();
///
/// assert_eq!(name.as_str(), "foo.rs");
/// ```
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone)]
pub struct FileName(String);

impl FileName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for FileName {
    type Err = MinigitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_file_name
            .parse(s.as_bytes())
            .map_err(|err| MinigitError::ParserError(err.to_string(), ParserContext::None))
    }
}

impl Display for FileName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<FileName> for PathBuf {
    fn from(val: FileName) -> Self {
        PathBuf::from(val.0)
    }
}

impl WriteLoose for FileName {
    fn write_loose<W: std::io::prelude::Write>(&self, writer: &mut W) -> Result<(), MinigitError> {
        write!(writer, "{}", self.0)?;

        Ok(())
    }
}

/// Parses a file name from some UTF-8 encoded bytes.
pub fn parse_file_name(input: &mut &[u8]) -> ModalResult<FileName> {
    terminated(take_until(1.., "\x00"), "\x00")
        .map(str::from_utf8)
        .verify_map(Result::ok)
        .map(str::to_owned)
        .map(FileName)
        .parse_next(input)
}

#[cfg(test)]
mod test {

    use std::assert_matches;

    use winnow::{Parser, error::ErrMode};

    use crate::fs::parse_file_name;

    #[test]
    fn file_name_parses_valid_inputs() {
        assert_eq!(
            parse_file_name
                .parse_peek(b"foo.bar\0")
                .unwrap()
                .1
                .to_string(),
            "foo.bar"
        );

        assert_eq!(
            parse_file_name
                .parse_peek(b"even this!!!\0")
                .unwrap()
                .1
                .to_string(),
            "even this!!!"
        );
    }

    #[test]
    fn file_name_rejects_invalid_inputs() {
        assert_matches!(
            parse_file_name.parse_peek(b"foo.bar"),
            Err(ErrMode::Backtrack(_))
        );
        assert_matches!(
            parse_file_name.parse_peek(b"foo.bar\n"),
            Err(ErrMode::Backtrack(_))
        );
    }
}
