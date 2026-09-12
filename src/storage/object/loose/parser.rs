/// This module deals with loose objects, i.e. objects in `.git/objects/`.
use winnow::{
    ModalResult, Parser,
    ascii::dec_uint,
    combinator::{alt, seq},
    error::{ContextError, ErrMode},
    token::literal,
};

use crate::object::ObjectType;

/// Parses an object type string from some bytes.
pub fn object_type(input: &mut &[u8]) -> ModalResult<ObjectType> {
    alt((
        literal("blob").map(|_| ObjectType::Blob),
        literal("tree").map(|_| ObjectType::Tree),
    ))
    .parse_next(input)
}

/// Given an input (which it consumes), read the object type and size of the rest of the object
pub fn header(input: &mut &[u8]) -> ModalResult<(ObjectType, usize)> {
    let mut size = dec_uint::<_, usize, ErrMode<ContextError>>;

    seq!(object_type, _: " ", size, _: "\0").parse_next(input)
}

#[cfg(test)]
mod tests {
    use crate::{
        object::ObjectType,
        storage::object::loose::parser::{header, object_type},
    };
    use std::assert_matches;
    use winnow::{Parser, error::ErrMode};

    #[test]
    fn object_type_parses_expected_object_types() {
        assert_eq!(
            object_type.parse_peek(b"blob"),
            Ok((&b""[..], ObjectType::Blob))
        );
        assert_eq!(
            object_type.parse_peek(b"tree"),
            Ok((&b""[..], ObjectType::Tree))
        );
    }

    #[test]
    fn object_type_rejects_invalid_input() {
        assert_matches!(object_type.parse_peek(b""), Err(ErrMode::Backtrack(_)));
        assert_matches!(object_type.parse_peek(b"blo"), Err(ErrMode::Backtrack(_)));
    }

    #[test]
    fn header_parses_basic_headers() {
        assert_eq!(
            header.parse_peek(b"blob 3\0"),
            Ok((&b""[..], (ObjectType::Blob, 3)))
        );
        assert_eq!(
            header.parse_peek(b"tree 333\0"),
            Ok((&b""[..], (ObjectType::Tree, 333)))
        );
    }

    #[test]
    fn header_rejets_invalid_input() {
        assert_matches!(header.parse_peek(b"tre 3"), Err(ErrMode::Backtrack(_)));
        assert_matches!(header.parse_peek(b"tree "), Err(ErrMode::Backtrack(_)));
        assert_matches!(header.parse_peek(b"tree \0"), Err(ErrMode::Backtrack(_)));
        assert_matches!(header.parse_peek(b"tree\0"), Err(ErrMode::Backtrack(_)));
        assert_matches!(header.parse_peek(b"3"), Err(ErrMode::Backtrack(_)));
        assert_matches!(header.parse_peek(b"3\0"), Err(ErrMode::Backtrack(_)));
    }
}
