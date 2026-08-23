use std::{fmt::Display, str::FromStr};

use anyhow::{Context, anyhow};
use sha1::digest::{array::Array, consts::U20};

/// A hash that identifies an [`crate::object::Object`].
pub struct ObjectHash(Array<u8, U20>);

impl ObjectHash {
    /// Gets the prefix (first byte) of the hash.
    pub fn prefix(&self) -> HashPrefix {
        self.0[0].into()
    }
}

impl From<Array<u8, U20>> for ObjectHash {
    fn from(value: Array<u8, U20>) -> Self {
        ObjectHash(value)
    }
}

impl Display for ObjectHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl TryFrom<&str> for ObjectHash {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let decoded = hex::decode(value)?;
        let bytes: [u8; 20] = decoded
            .try_into()
            .map_err(|_| anyhow!("couldn't turn hash into a 20-byte array"))?;

        Ok(ObjectHash(bytes.into()))
    }
}

impl<'a> TryFrom<std::borrow::Cow<'a, str>> for ObjectHash {
    type Error = anyhow::Error;

    fn try_from(value: std::borrow::Cow<'a, str>) -> Result<Self, Self::Error> {
        Self::try_from(value.as_ref())
    }
}

impl TryFrom<String> for ObjectHash {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

/// Prefix (first byte) of an [ObjectHash].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct HashPrefix(u8);

impl Display for HashPrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:2x}", self.0)
    }
}

impl From<u8> for HashPrefix {
    fn from(value: u8) -> Self {
        HashPrefix(value)
    }
}

impl TryFrom<&str> for HashPrefix {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let decoded = hex::decode(value)?;
        let first_byte = *(decoded.first().context("couldn't get first decoded byte")?);

        Ok(HashPrefix(first_byte))
    }
}

impl<'a> TryFrom<std::borrow::Cow<'a, str>> for HashPrefix {
    type Error = anyhow::Error;

    fn try_from(value: std::borrow::Cow<'a, str>) -> Result<Self, Self::Error> {
        Self::try_from(value.as_ref())
    }
}

impl FromStr for HashPrefix {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl TryFrom<String> for HashPrefix {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

#[cfg(test)]
mod test {
    use crate::hash::HashPrefix;

    #[test]
    fn test_hash_prefix_display() {
        assert_eq!(format!("{}", HashPrefix(0x00)), "00".to_string());
        assert_eq!(format!("{}", HashPrefix(0x2f)), "2f".to_string());
    }
}
