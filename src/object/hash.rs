//! Types and functions dealing with hashing of objects.
//!
//! In the loose object store (`.git/objects`), objects are indexed by their [ObjectHash]
//! represented in hexadecimal. In particular, they are placed in buckets named after the
//! [HashPrefix], (i.e. the first two characters of the hexadecimal hash), and the prefix is removed
//! from the object file name.
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
use std::{
    fmt::Display,
    path::PathBuf,
    str::FromStr,
};

use anyhow::{Context, anyhow};
use sha1::digest::{array::Array, consts::U20};

/// A hash that identifies an [`super::Object`]. It is a SHA1 hash of the header and
/// contents of the object.
#[derive(Debug, PartialEq)]
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

impl FromStr for ObjectHash {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let decoded = hex::decode(value)?;
        let bytes: [u8; 20] = decoded
            .try_into()
            .map_err(|_| anyhow!("couldn't turn hash into a 20-byte array"))?;

        Ok(ObjectHash(bytes.into()))
    }
}

impl TryFrom<&PathBuf> for ObjectHash {
    type Error = anyhow::Error;

    /// Converts a [PathBuf] into an [ObjectHash]. This expects a path to a loose object that is
    /// under a bucket directory.
    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        let prefix = value
            .parent()
            .context("couldn't get parent directory")?
            .file_name()
            .context("couldn't get parent directory name")?
            .to_string_lossy();

        let name = value
            .file_name()
            .context("couldn't get file name")?
            .to_string_lossy();

        let hash_str = format!("{prefix}{name}");

        hash_str.parse()
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

impl FromStr for HashPrefix {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decoded = hex::decode(s)?;
        let first_byte = *(decoded.first().context("couldn't get first decoded byte")?);

        Ok(HashPrefix(first_byte))
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use crate::object::hash::{HashPrefix, ObjectHash};

    #[test]
    fn test_parse_objecthash_try_from_path() {
        let path = "/home/alex/minigit/.git/objects/9d/5476d96d3262b69f03f2af27750a495cca43b6"
            .parse::<PathBuf>()
            .unwrap();

        assert_eq!(
            ObjectHash::try_from(&path).expect("can't parse PathBuf into ObjectHash"),
            ObjectHash(
                [
                    0x9d, 0x54, 0x76, 0xd9, 0x6d, 0x32, 0x62, 0xb6, 0x9f, 0x03, 0xf2, 0xaf, 0x27,
                    0x75, 0x0a, 0x49, 0x5c, 0xca, 0x43, 0xb6
                ]
                .into()
            )
        );
    }

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
