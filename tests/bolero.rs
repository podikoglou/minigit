use bolero::check;
use chrono::{DateTime, FixedOffset};
use minigit::{
    error::ParserContext,
    fs::parse_file_name,
    identity::parse_identity,
    object::{
        commit::parse_commit,
        parse_header,
        tag::parse_tag,
        tree::parse_tree,
    },
    parsing::Stream,
    storage::object::loose::{WriteLoose, read_object, read_object_compressed},
    time::{Timestamp, parse_timestamp},
};

/// Fuzz the uncompressed loose object parser with arbitrary raw bytes.
/// Whenever an input successfully parses into a valid Minigit `Object`,
/// verify that serializing it and re-parsing it is completely lossless (roundtrip).
#[test]
fn fuzz_read_loose_object() {
    check!().for_each(|input: &[u8]| {
        if let Ok(object) = read_object(input, ParserContext::None) {
            let mut serialized = Vec::new();
            if object.write_loose(&mut serialized).is_ok() {
                let roundtripped = read_object(&serialized[..], ParserContext::None)
                    .expect("roundtripped object should parse successfully");
                assert_eq!(object, roundtripped);
            }
        }
    });
}

/// Fuzz the zlib-compressed loose object parser with arbitrary bytes to ensure
/// neither flate2/zlib nor winnow panics on corrupt or malformed payloads.
#[test]
fn fuzz_read_loose_object_compressed() {
    check!().for_each(|input: &[u8]| {
        let _ = read_object_compressed(input, ParserContext::None);
    });
}

/// Fuzz the git object header parser (`<type> <size>\0`).
#[test]
fn fuzz_header() {
    check!().for_each(|input: &[u8]| {
        let mut stream = Stream::new(input);
        let _ = parse_header(&mut stream);
    });
}

/// Fuzz the raw timestamp parser on arbitrary byte sequences.
#[test]
fn fuzz_timestamp() {
    check!().for_each(|input: &[u8]| {
        let mut stream = Stream::new(input);
        let _ = parse_timestamp(&mut stream);
    });
}

/// Property test for Timestamp using structured generation:
/// Generate valid (unix_secs, offset_hours, offset_minutes) directly,
/// format them into Git timestamp wire format, and verify roundtrip parsing.
#[test]
fn prop_timestamp_roundtrip() {
    let secs = 0i64..2_000_000_000i64;
    let offset_hours = -12i32..=14i32;
    let offset_minutes = 0i32..60i32;

    check!()
        .with_generator((secs, offset_hours, offset_minutes))
        .for_each(|&(s, h, m)| {
            let total_offset_secs = h * 3600 + if h >= 0 { m * 60 } else { -m * 60 };
            if let Some(offset) = FixedOffset::east_opt(total_offset_secs) {
                if let Some(dt) = DateTime::from_timestamp(s, 0) {
                    let dt = dt.with_timezone(&offset);
                    if let Ok(ts) = Timestamp::try_new(dt) {
                        let sign = if h >= 0 { '+' } else { '-' };
                        let formatted = format!("{} {}{:02}{:02}", ts.timestamp(), sign, h.abs(), m);
                        let mut stream = Stream::new(formatted.as_bytes());
                        let parsed = parse_timestamp(&mut stream)
                            .expect("validly formatted timestamp should parse");
                        assert_eq!(parsed, ts);
                    }
                }
            }
        });
}

/// Fuzz the identity parser (`Name <email>`).
#[test]
fn fuzz_identity() {
    check!().for_each(|input: &[u8]| {
        let mut stream = Stream::new(input);
        let _ = parse_identity(&mut stream);
    });
}

/// Fuzz the null-terminated filename parser.
#[test]
fn fuzz_file_name() {
    check!().for_each(|input: &[u8]| {
        let mut stream = Stream::new(input);
        let _ = parse_file_name(&mut stream);
    });
}

/// Fuzz the commit object parser.
#[test]
fn fuzz_commit() {
    check!().for_each(|input: &[u8]| {
        let mut stream = Stream::new(input);
        let _ = parse_commit(&mut stream);
    });
}

/// Fuzz the tag object parser.
#[test]
fn fuzz_tag() {
    check!().for_each(|input: &[u8]| {
        let mut stream = Stream::new(input);
        let _ = parse_tag(&mut stream);
    });
}

/// Fuzz the tree object parser.
#[test]
fn fuzz_tree() {
    check!().for_each(|input: &[u8]| {
        let mut stream = Stream::new(input);
        let _ = parse_tree(&mut stream);
    });
}
