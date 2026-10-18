use std::{fs::File, path::PathBuf, str::FromStr};

use memmap2::Mmap;
use winnow::{
    ModalResult, Parser,
    ascii::{newline, space0, till_line_ending},
    combinator::{alt, opt, preceded, repeat, seq},
    error::StrContext,
    token::take,
};

use crate::{
    MinigitError,
    error::ParserContext,
    object::hash::{ObjectHash, parse_object_hash_str},
    parsing::Stream,
};

/// Provides a view over a `packed-refs` database file
pub struct RefsDb {
    pub path: PathBuf,
    file: File,
    mmap: Mmap,
}

impl RefsDb {
    /// Opens a `packed-refs` file
    pub fn open(path: PathBuf) -> Result<Self, MinigitError> {
        let file = File::open(&path)?;
        let mmap = unsafe { Mmap::map(&file) }?;

        parse_header
            .parse_next(&mut Stream::new(&mmap))
            .map_err(|err| MinigitError::ParserError(err.to_string(), ParserContext::None))?;

        Ok(Self { path, file, mmap })
    }

    /// Looks up a ref's unpeeled hash
    pub fn lookup(&self, name: &str) -> Result<RefsDbEntry, MinigitError> {
        // This is of couse an inefficient implementation. This does no binary search whatsoever,
        // despite the file being sorted.
        //
        // TODO: implement this using binary search

        let mut stream = Stream::new(&self.mmap);

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

#[derive(Debug)]
pub struct RefsDbEntry {
    /// A hash that points to an object, oftentimes a tag.
    pub hash: ObjectHash,

    pub name: String,

    /// A hash that points directly to the commit object.
    pub peeled_hash: Option<ObjectHash>,
}

impl RefsDbEntry {
    pub fn is_peeled(&self) -> bool {
        self.peeled_hash.is_some()
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

fn parse_entry(input: &mut Stream<'_>) -> ModalResult<RefsDbEntry> {
    seq! {RefsDbEntry {
        hash: parse_object_hash_str.context(StrContext::Label("pointer hash")),
        _: " ",
        name: terminated(till_line_ending, opt(newline)).map(str::from_utf8).verify_map(Result::ok).map(str::to_owned).context(StrContext::Label("ref name")),
        peeled_hash: opt(seq!(_: "^", parse_object_hash_str, _: newline).map(|x| x.0)).context(StrContext::Label("peeled pointer hash")),
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
