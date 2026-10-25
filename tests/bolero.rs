use bolero::check;
use chrono::{DateTime, FixedOffset};
use minigit::{
    error::ParserContext,
    fs::parse_file_name,
    identity::parse_identity,
    object::{commit::parse_commit, parse_header, tag::parse_tag, tree::parse_tree},
    parsing::Stream,
    storage::object::{
        loose::{Encode, read_object, read_object_compressed},
        packed::{
            PackedObjectType, base_object,
            delta::{copy_instruction, delta, insert_instruction, instruction},
            object_header, size, undeltified_object,
        },
    },
    time::{Timestamp, parse_timestamp},
};
use winnow::Parser;

/// Fuzz the uncompressed loose object parser with arbitrary raw bytes.
/// Whenever an input successfully parses into a valid Minigit `Object`,
/// verify that serializing it and re-parsing it is completely lossless (roundtrip).
#[test]
fn fuzz_read_loose_object() {
    check!().for_each(|input: &[u8]| {
        if let Ok(object) = read_object(input, ParserContext::None) {
            let mut serialized = Vec::new();
            if object.encode(&mut serialized).is_ok() {
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
                        let formatted =
                            format!("{} {}{:02}{:02}", ts.timestamp(), sign, h.abs(), m);
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

/// Fuzz the insert instruction parser and writer (roundtrip)
#[test]
fn fuzz_insert_instruction() {
    check!().for_each(|input: &[u8]| {
        let stream = Stream::new(input);

        if let Ok(instruction) = insert_instruction.parse(stream) {
            let mut buf = Vec::default();

            let _ = instruction.encode(&mut buf);

            assert_eq!(buf, input);
        }
    });
}

/// Fuzz the copy instruction parser and writer (roundtrip)
///
/// This property is a little different than others. The flow is:
/// 1. Get input data
/// 2. Parse it
/// 3. Encode it to bytes
/// 3. Parse it
/// 4. Compare the two parsed versions
///
/// This is because copy instructions speciically are not bijective, i.e. there can be multiple raw
/// bytes that map to the same instruction representation, and our encoder isn't guaranteed to
/// output the exact same raw bytes as we were given, despite them being correct and parsing to the
/// same instruction.
#[test]
fn fuzz_copy_instruction() {
    check!().for_each(|input: &[u8]| {
        let stream = Stream::new(input);

        if let Ok(instruction) = copy_instruction.parse(stream) {
            let mut buf = Vec::default();

            let _ = instruction.encode(&mut buf);

            let stream_2 = Stream::new(&buf);

            let instruction_2 = copy_instruction
                .parse(stream_2)
                .expect("should be parseable");
            assert_eq!(instruction_2, instruction);
        }
    });
}

#[test]
fn fuzz_delta_instruction() {
    check!().for_each(|input: &[u8]| {
        let stream = Stream::new(input);

        if let Ok(instr) = instruction.parse(stream) {
            let mut buf = Vec::default();

            let _ = instr.encode(&mut buf);

            let stream_2 = Stream::new(&buf);

            let instruction_2 = instruction.parse(stream_2).expect("should be parseable");
            assert_eq!(instruction_2, instr);
        }
    });
}

#[test]
fn fuzz_size() {
    check!().for_each(|input: &[u8]| {
        let stream = Stream::new(input);

        if let Ok(parsed) = size.parse(stream) {
            let mut buf = Vec::default();

            (parsed as usize)
                .encode(&mut buf)
                .expect("should be encodable");

            let parsed_2 = size.parse(Stream::new(&buf)).expect("should be parseable");

            assert_eq!(parsed_2, parsed);
        }
    });
}

#[test]
fn fuzz_delta() {
    check!().for_each(|input: &[u8]| {
        let stream = Stream::new(input);

        if let Ok(parsed) = delta.parse(stream) {
            let mut buf = Vec::default();

            (parsed).encode(&mut buf).expect("should be encodable");

            let parsed_2 = delta.parse(Stream::new(&buf)).expect("should be parseable");

            assert_eq!(parsed_2, parsed);
        }
    });
}

#[test]
fn fuzz_undeltified_object() {
    check!().for_each(|input: &[u8]| {
        let stream = Stream::new(input);

        if let Ok(parsed) = undeltified_object.parse(stream) {
            let mut buf = Vec::default();

            parsed.encode(&mut buf).expect("should be encodable");

            let parsed_2 = undeltified_object
                .parse(Stream::new(&buf))
                .expect("should be parseable");

            assert_eq!(parsed_2, parsed);
        }
    });
}

#[test]
fn fuzz_ofs_delta_base_object() {
    check!().for_each(|input: &[u8]| {
        let stream = Stream::new(input);

        if let Ok(parsed) = base_object(PackedObjectType::OfsDelta).parse(stream) {
            let mut buf = Vec::default();

            parsed.encode(&mut buf).expect("should be encodable");

            let parsed_2 = base_object(PackedObjectType::OfsDelta)
                .parse(Stream::new(&buf))
                .expect("should be parseable");

            assert_eq!(parsed_2, parsed);
        }
    });
}

#[test]
fn fuzz_ref_delta_base_object() {
    check!().for_each(|input: &[u8]| {
        let stream = Stream::new(input);

        if let Ok(parsed) = base_object(PackedObjectType::RefDelta).parse(stream) {
            let mut buf = Vec::default();

            parsed.encode(&mut buf).expect("should be encodable");

            let parsed_2 = base_object(PackedObjectType::RefDelta)
                .parse(Stream::new(&buf))
                .expect("should be parseable");

            assert_eq!(parsed_2, parsed);
        }
    });
}

#[test]
fn fuzz_object_header() {
    check!().for_each(|input: &[u8]| {
        let stream = Stream::new(input);

        if let Ok(parsed) = object_header.parse(stream) {
            let mut buf = Vec::default();

            parsed.encode(&mut buf).expect("should be encodable");

            let parsed_2 = object_header
                .parse(Stream::new(&buf))
                .expect("should be parseable");

            assert_eq!(parsed_2, parsed);
        }
    });
}
