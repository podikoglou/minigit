use std::{fs::File, path::PathBuf, str::FromStr};

use memmap2::Mmap;
use winnow::{
    ModalResult, Parser,
    ascii::{newline, space0, till_line_ending},
    combinator::{alt, opt, preceded, repeat, seq, terminated},
    error::StrContext,
    token::take,
};

use crate::{
    MinigitError,
    error::ParserContext,
    object::hash::{ObjectHash, parse_object_hash_str},
    parsing::Stream,
    storage::reference::{Ref, RefPointer},
};

/// Provides a view over a `packed-refs` database file
pub struct RefsDb {
    pub path: PathBuf,
    pub flags: Flags,
    file: File,
    mmap: Mmap,
}

impl RefsDb {
    /// Opens a `packed-refs` file
    pub fn open(path: PathBuf) -> Result<Self, MinigitError> {
        let file = File::open(&path)?;
        let mmap = unsafe { Mmap::map(&file) }?;

        let flags = parse_header
            .parse_next(&mut Stream::new(&mmap))
            .map_err(|err| MinigitError::ParserError(err.to_string(), ParserContext::None))?;

        Ok(Self {
            path,
            flags,
            file,
            mmap,
        })
    }

    /// Looks up a ref's unpeeled hash
    pub fn lookup(&self, name: &str) -> Result<Ref, MinigitError> {
        // This is of couse an inefficient implementation. This does no binary search whatsoever,
        // despite the file being sorted.
        //
        // TODO: implement this using binary search

        let mut stream = Stream::new(&self.mmap);

        terminated(parse_header, opt(newline))
            .parse_next(&mut stream)
            .map_err(|err| MinigitError::ParserError(err.to_string(), ParserContext::None))?;

        while !stream.is_empty() {
            let entry = parse_entry
                .parse_next(&mut stream)
                .map_err(|err| MinigitError::ParserError(err.to_string(), ParserContext::None))?;

            if entry.name == name {
                return Ok(entry);
            }
        }

        Err(MinigitError::RefNotFound)
    }
}

#[derive(Debug, PartialEq, Eq, Default, Clone, Copy)]
pub struct Flags {
    pub peeled: bool,
    pub fully_peeled: bool,
    pub sorted: bool,
}

fn flags(input: &mut Stream<'_>) -> ModalResult<Flags> {
    let mut flags = Flags::default();
    let parsed: Vec<&[u8]> = repeat(
        0..,
        preceded(space0, alt(("peeled", "fully-peeled", "sorted"))),
    )
    .parse_next(input)?;
    let _ = space0.parse_next(input)?;

    for flag in parsed {
        match flag {
            b"peeled" => flags.peeled = true,
            b"fully-peeled" => flags.fully_peeled = true,
            b"sorted" => flags.sorted = true,
            _ => {}
        }
    }

    Ok(flags)
}

fn parse_header(input: &mut Stream<'_>) -> ModalResult<Flags> {
    preceded("# pack-refs with:", flags)
        .context(StrContext::Label("packed-refs header"))
        .parse_next(input)
}

fn parse_entry(input: &mut Stream<'_>) -> ModalResult<Ref> {
    seq! {Ref {
        pointer: parse_object_hash_str.context(StrContext::Label("pointer hash")).map(RefPointer::Hash),
        _: " ",
        name: terminated(till_line_ending, opt(newline)).map(str::from_utf8).verify_map(Result::ok).map(str::to_owned).context(StrContext::Label("ref name")),
        peeled: opt(seq!(_: "^", parse_object_hash_str, _: newline).map(|x| x.0)).context(StrContext::Label("peeled pointer hash")),
    }}
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;
    use winnow::Parser;

    use crate::{parsing::Stream, storage::reference::db::parse_header};

    #[test]
    fn parses_header_flags() {
        assert_debug_snapshot!(parse_header.parse_peek(Stream::new(b"# pack-refs with:")));
        assert_debug_snapshot!(parse_header.parse_peek(Stream::new(b"# pack-refs with: ")));
        assert_debug_snapshot!(parse_header.parse_peek(Stream::new(b"# pack-refs with: peeled")));
        assert_debug_snapshot!(parse_header.parse_peek(Stream::new(b"# pack-refs with:  peeled")));
        assert_debug_snapshot!(
            parse_header.parse_peek(Stream::new(b"# pack-refs with: fully-peeled"))
        );
        assert_debug_snapshot!(parse_header.parse_peek(Stream::new(b"# pack-refs with: sorted")));
        assert_debug_snapshot!(
            parse_header.parse_peek(Stream::new(b"# pack-refs with: sorted fully-peeled peeled"))
        );
        assert_debug_snapshot!(
            parse_header.parse_peek(Stream::new(b"# pack-refs with: fully-peeled sorted"))
        );
    }
}
