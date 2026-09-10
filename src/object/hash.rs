//! Types and functions dealing with hashing of objects.
//!
//! In the git object store (`.git/objects`), objects are indexed by their [ObjectHash] represented
//! in hexadecimal. In particular they are placed in buckets named after the [HashPrefix], (i.e. the
//! first two characters of the hexadecimal hash), and the prefix is removed from the object file
//! name.
//!
//! ```txt
//! .git/objects
//! ├── 00
//! │   ├── 77a275f2a44ea4c1ea187e9bbb95998a468e43
//! │   ├── 87288858ff994e811024ebe37e0035fafad790
//! │   ├── f856dd6c92aec1cbd77b2204cf409d47580cb5
//! │   └
//! ```
//!
//! In this objects store for example, there exist three objects with the following hashes:
//! - `0077a275f2a44ea4c1ea187e9bbb95998a468e43`
//! - `0087288858ff994e811024ebe37e0035fafad790`
//! - `00f856dd6c92aec1cbd77b2204cf409d47580cb5`
use std::{fmt::Display, str::FromStr};

use anyhow::{Context, anyhow};
use sha1::digest::{array::Array, consts::U20};

/// A hash that identifies an [`super::Object`]. It is a SHA1 hash of the header and
/// contents of the object.
pub struct ObjectHash(Array<u8, U20>);

impl ObjectHash {
    /// Gets the [HashPrefix] (first byte) of the hash.
    pub fn prefix(&self) -> HashPrefix {
        self.0[0].into()
    }
}

impl From<Array<u8, U20>> for ObjectHash {
    fn from(value: Array<u8, U20>) -> Self {
        ObjectHash(value)
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

impl Display for ObjectHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

/// Prefix of an [ObjectHash]. This is the first byte of the hash.
///
/// This is used in the object store for indexing objects by the first byte of their hash.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct HashPrefix(u8);

impl Display for HashPrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02x}", self.0)
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
    use crate::object::hash::{HashPrefix, ObjectHash};

    #[test]
    fn test_hash_prefix_display() {
        assert_eq!(format!("{}", HashPrefix(0x00)), "00".to_string());
        assert_eq!(format!("{}", HashPrefix(0x2f)), "2f".to_string());
        assert_eq!(format!("{}", HashPrefix(0x10)), "10".to_string());
    }

    #[test]
    fn test_hash_display() {
        assert_eq!(
            format!(
                "{}",
                ObjectHash(
                    [
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
                    ]
                    .into()
                )
            ),
            "0000000000000000000000000000000000000000".to_string()
        );

        assert_eq!(
            format!(
                "{}",
                ObjectHash(
                    [
                        0xdb, 0xc7, 0x4b, 0x22, 0x44, 0xf5, 0x7e, 0xd7, 0xdd, 0xf6, 0xe5, 0xb5,
                        0x3c, 0x17, 0x82, 0x6b, 0xfb, 0xf9, 0xbe, 0x51
                    ]
                    .into()
                )
            ),
            "dbc74b2244f57ed7ddf6e5b53c17826bfbf9be51".to_string()
        );
    }
}
