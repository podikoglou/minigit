//! This module contains items that deal with file names.

use std::{
    fmt::{Display, Write},
    path::PathBuf,
    str::FromStr,
};

use crate::MinigitError;

/// A file name.
///
/// This is a newtype which is validated upon creation, thus a [`FileName`] is assumed to be a valid
/// file name that Git supports.
///
/// A [`FileName`] can be created using [`FromStr`]. For example:
/// ```
/// use minigit::fs::FileName;
///
/// let name: FileName = "foo.rs".parse().unwrap();
///
/// assert_eq!(name.as_str(), "foo.rs");
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FileName(String);

impl FileName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for FileName {
    type Err = MinigitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: actually do validation
        Ok(FileName(s.to_owned()))
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
