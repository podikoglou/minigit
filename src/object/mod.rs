pub mod blob;
pub mod commit;
pub mod tag;
pub mod tree;

pub mod hash;

use std::io::Write;

use blob::Blob;
use flate2::{Compression, write::ZlibEncoder};
use sha1::{Digest, Sha1};
use strum::{EnumDiscriminants, EnumString, IntoStaticStr, VariantArray};
use tree::Tree;
use winnow::{
    ModalResult, Parser,
    ascii::dec_uint,
    combinator::{alt, seq},
    error::{ContextError, ErrMode, StrContext, StrContextValue},
    token::{literal, take},
};

use crate::{
    MinigitError,
    error::ParserContext,
    object::{
        blob::parse_blob,
        commit::{Commit, parse_commit},
        hash::ObjectHash,
        tag::{Tag, parse_tag},
        tree::parse_tree,
    },
    parsing::Stream,
    storage::object::{LazyObject, loose::WriteLoose},
};

#[derive(Debug, PartialEq, Eq, Clone, EnumDiscriminants)]
#[strum_discriminants(name(ObjectType))]
#[strum_discriminants(derive(EnumString, IntoStaticStr, VariantArray))]
#[strum_discriminants(strum(ascii_case_insensitive))]
#[strum_discriminants(strum(serialize_all = "lowercase"))]
pub enum Object {
    Blob(Blob),
    Tree(Tree),
    Commit(Box<Commit>),
    Tag(Tag),
}

impl Object {
    /// Creates a SHA1 hash of the object.
    pub fn hash(&self) -> Result<ObjectHash, MinigitError> {
        let mut buf = Vec::new();
        self.write_loose(&mut buf)?;

        Ok(Sha1::digest(buf).into())
    }
}

impl WriteLoose for Object {
    /// Writes the uncompressed object to a write.
    fn write_loose<W: Write>(&self, writer: &mut W) -> Result<(), crate::MinigitError> {
        let mut buf: Vec<u8> = Vec::new();

        match self {
            Object::Blob(blob) => {
                write!(writer, "blob ")?;

                blob.write_loose(&mut buf)?;
            }
            Object::Tree(tree) => {
                write!(writer, "tree ")?;

                tree.write_loose(&mut buf)?;
            }
            Object::Commit(commit) => {
                write!(writer, "commit ")?;

                commit.write_loose(&mut buf)?;
            }
            Object::Tag(tag) => {
                write!(writer, "tag ")?;

                tag.write_loose(&mut buf)?;
            }
        }

        write!(writer, "{}\0", buf.len())?;
        writer.write_all(&buf)?;

        Ok(())
    }
}

impl Object {
    pub fn write_loose_compressed<W: Write>(
        &self,
        writer: &mut W,
    ) -> Result<(), crate::MinigitError> {
        let mut zlib_writer = ZlibEncoder::new(writer, Compression::default());

        self.write_loose(&mut zlib_writer)
    }
}

impl From<Blob> for Object {
    fn from(val: Blob) -> Self {
        Self::Blob(val)
    }
}

impl From<Tree> for Object {
    fn from(val: Tree) -> Self {
        Self::Tree(val)
    }
}

impl From<Commit> for Object {
    fn from(val: Commit) -> Self {
        Self::Commit(Box::new(val))
    }
}
impl From<Tag> for Object {
    fn from(val: Tag) -> Self {
        Self::Tag(val)
    }
}

impl TryFrom<LazyObject> for Object {
    type Error = MinigitError;

    fn try_from(value: LazyObject) -> Result<Self, Self::Error> {
        value.into_object()
    }
}

/// Parses an [Object] from some bytes.
///
/// Unless you're building your own parsers this is the function you're looking for.
pub fn parse_object(input: &[u8], context: ParserContext) -> Result<Object, MinigitError> {
    object
        .parse(input)
        .map_err(|err| MinigitError::ParserError(err.to_string(), context))
}

/// Parses an [Object] from some input.
pub fn object<'a>(input: &mut Stream<'a>) -> ModalResult<Object> {
    let (typee, size) = parse_header.parse_next(input)?;
    let mut bytes: Stream<'a> = take(size).parse_next(input)?;

    match typee {
        ObjectType::Blob => parse_blob.map(Object::Blob).parse_next(&mut bytes),
        ObjectType::Tree => parse_tree.map(Object::Tree).parse_next(&mut bytes),
        ObjectType::Commit => parse_commit.map(Object::from).parse_next(&mut bytes),
        ObjectType::Tag => parse_tag.map(Object::from).parse_next(&mut bytes),
    }
}

/// Parse a header (object type and size) from some bytes.
pub fn parse_header<'a>(input: &mut Stream<'a>) -> ModalResult<(ObjectType, usize)> {
    let mut size = dec_uint::<_, usize, ErrMode<ContextError>>
        .context(StrContext::Label("payload size"))
        .context(StrContext::Expected(StrContextValue::Description(
            "bytes amount",
        )));

    seq!(parse_object_type, _: " ", size, _: "\0")
        .context(StrContext::Label("header"))
        .parse_next(input)
}

/// Parses an object type string from some bytes.
pub fn parse_object_type<'a>(input: &mut Stream<'a>) -> ModalResult<ObjectType> {
    alt((
        literal("blob").value(ObjectType::Blob),
        literal("tree").value(ObjectType::Tree),
        literal("commit").value(ObjectType::Commit),
        literal("tag").value(ObjectType::Tag),
    ))
    .context(StrContext::Label("type"))
    .context(StrContext::Expected(StrContextValue::Description(
        "blob | tree | commit | tag",
    )))
    .parse_next(input)
}

#[cfg(test)]
mod test {
    use std::assert_matches;

    use winnow::{Parser, error::ErrMode};

    use crate::object::{ObjectType, parse_header, parse_object_type};

    #[test]
    fn object_type_parses_expected_object_types() {
        assert_eq!(
            parse_object_type.parse_peek(b"blob"),
            Ok((&b""[..], ObjectType::Blob))
        );
        assert_eq!(
            parse_object_type.parse_peek(b"tree"),
            Ok((&b""[..], ObjectType::Tree))
        );
    }

    #[test]
    fn object_type_rejects_invalid_input() {
        assert_matches!(
            parse_object_type.parse_peek(b""),
            Err(ErrMode::Backtrack(_))
        );
        assert_matches!(
            parse_object_type.parse_peek(b"blo"),
            Err(ErrMode::Backtrack(_))
        );
    }

    #[test]
    fn header_parses_basic_headers() {
        assert_eq!(
            parse_header.parse_peek(b"blob 3\0"),
            Ok((&b""[..], (ObjectType::Blob, 3)))
        );
        assert_eq!(
            parse_header.parse_peek(b"tree 333\0"),
            Ok((&b""[..], (ObjectType::Tree, 333)))
        );
    }

    #[test]
    fn header_rejets_invalid_input() {
        assert_matches!(
            parse_header.parse_peek(b"tre 3"),
            Err(ErrMode::Backtrack(_))
        );
        assert_matches!(
            parse_header.parse_peek(b"tree "),
            Err(ErrMode::Backtrack(_))
        );
        assert_matches!(
            parse_header.parse_peek(b"tree \0"),
            Err(ErrMode::Backtrack(_))
        );
        assert_matches!(
            parse_header.parse_peek(b"tree\0"),
            Err(ErrMode::Backtrack(_))
        );
        assert_matches!(parse_header.parse_peek(b"3"), Err(ErrMode::Backtrack(_)));
        assert_matches!(parse_header.parse_peek(b"3\0"), Err(ErrMode::Backtrack(_)));
    }
}
