use std::assert_matches;
use std::fs::File;

use memmap2::Mmap;
use minigit::{
    Repo,
    object::Object,
    parsing::Stream,
    storage::object::packed::{
        self, BaseObject, CopyInstruction, Delta, Instruction, PackedObject, Packfile,
        idx::PackIndex,
    },
};
use winnow::Parser;

mod common;

#[test]
fn undeltified_packfile_parse() {
    let (_, dir) = include_repo!("fixtures/repo-5.tar");

    let path = dir
        .path()
        .join(".git/objects/pack/pack-a0ba98959a3d422d8bbe8e86f2cf62eb8f2b2df7.pack");

    let file = File::open(path).expect("should be able to open packfile");

    let mmap = unsafe { Mmap::map(&file).expect("should be able to mmap file") };
    let mut slice = Stream::new(&mmap[..]);

    let header = packed::header
        .parse_next(&mut slice)
        .expect("should be able to parse packfile header");

    assert_eq!(header.objects, 3, "should have three objects");

    // object 1
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(_, Object::Commit(_)));
    }

    // object 2
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(_, Object::Blob(_)));
    }

    // object 3
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(_, Object::Tree(_)));
    }
}

#[test]
fn deltified_packfile_parse() {
    let (_, dir) = include_repo!("fixtures/repo-6.tar");

    let path = dir
        .path()
        .join(".git/objects/pack/pack-c7e64c7164f8f92168a1ce0953f0716848c2c32c.pack");

    let file = File::open(path).expect("should be able to open packfile");

    let mmap = unsafe { Mmap::map(&file).expect("should be able to mmap file") };
    let mut slice = Stream::new(&mmap[..]);

    let header = packed::header
        .parse_next(&mut slice)
        .expect("should be able to parse packfile header");

    assert_eq!(header.objects, 12, "should have 12 objects");

    // object 1
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(_, Object::Commit(_)));
    }

    // object 2
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(_, Object::Commit(_)));
    }

    // object 3
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(_, Object::Commit(_)));
    }

    // object 4
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(_, Object::Commit(_)));
    }

    // object 5
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(_, Object::Blob(_)));
    }

    // object 6
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(_, Object::Blob(_)));
    }

    // object 7
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(_, Object::Tree(_)));
    }

    // object 8
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(_, Object::Tree(_)));
    }

    // object 9 (deltified)
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_eq!(
            object,
            PackedObject::Deltified {
                offset: 870,
                base: BaseObject::Ofs(870 - 264),
                delta: Delta {
                    base_object_size: 177,
                    deltified_object_size: 154,
                    instructions: vec![Instruction::Copy(CopyInstruction {
                        offset: 0,
                        size: 154,
                    })],
                },
            }
        );
    }

    // object 10
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(_, Object::Tree(_)));
    }

    // object 11
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(_, Object::Blob(_)));
    }

    // object 12
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(_, Object::Tree(_)));
    }

    assert_eq!(slice.len(), 20, "packfile should end with 20-byte checksum");
}

#[test]
fn idx_parse() {
    let (_, dir) = include_repo!("fixtures/repo-5.tar");

    let path = dir
        .path()
        .join(".git/objects/pack/pack-a0ba98959a3d422d8bbe8e86f2cf62eb8f2b2df7.idx");

    let file = File::open(path).expect("should be able to open idx file");

    let mmap = unsafe { Mmap::map(&file).expect("should be able to mmap file") };
    let mut slice = Stream::new(&mmap[..]);

    packed::idx::header
        .parse_next(&mut slice)
        .expect("should be able to parse header");

    let fanout_table = packed::idx::fanout_table
        .parse_next(&mut slice)
        .expect("should be able to parse fanout table");

    for (idx, window) in fanout_table.windows(2).enumerate() {
        assert!(
            window[0] <= window[1],
            "window {} should not break monotonicity",
            idx
        );
    }

    let entries = *fanout_table
        .last()
        .expect("fanout table should have last element") as usize;

    let object_ids = packed::idx::object_names(entries)
        .parse_next(&mut slice)
        .expect("should be able to parse object names");

    let crcs = packed::idx::crc_entries(entries)
        .parse_next(&mut slice)
        .expect("should be able to parse CRC entries");

    let offsets_1 = packed::idx::offsets_1(entries)
        .parse_next(&mut slice)
        .expect("should be able to parse first offset table");

    let offsets_2_entries = offsets_1
        .iter()
        .filter(|entry| (*entry & 0x8000_0000) != 0)
        .count();

    let _ = packed::idx::offsets_2(offsets_2_entries)
        .parse_next(&mut slice)
        .expect("should be able to parse second offset table");

    // TODO: checksums
}

#[test]
fn objects_iterator() {
    let (_, dir) = include_repo!("fixtures/repo-6.tar");

    let path = dir
        .path()
        .join(".git/objects/pack/pack-c7e64c7164f8f92168a1ce0953f0716848c2c32c.pack");

    let packfile = Packfile::open(path, None).expect("should be able to open packfile");
    let objects = packfile.objects();

    assert_eq!(
        objects.count(),
        packfile.objects_count as usize,
        "read objects should match packfile header's object count"
    );
}

#[test]
fn idx_read() {
    let (_, dir) = include_repo!("fixtures/repo-5.tar");

    let path = dir
        .path()
        .join(".git/objects/pack/pack-a0ba98959a3d422d8bbe8e86f2cf62eb8f2b2df7.idx");

    let idx = PackIndex::open(path).expect("should be able to open pack idx file");

    assert_eq!(
        idx.fanout_table.len(),
        256,
        "fanout table should have 256 entries",
    );

    for (idx, window) in idx.fanout_table.windows(2).enumerate() {
        assert!(
            window[0] <= window[1],
            "window {} should not break monotonicity",
            idx
        );
    }

    let objects_count = idx.objects_count();

    assert_eq!(
        objects_count, 3,
        "object count (last fanoutt entry) should be 3"
    );

    assert_eq!(
        idx.object_names.len(),
        objects_count,
        "object names count should match object count"
    );

    assert_eq!(
        idx.crc_entries.len(),
        objects_count,
        "crc entries count should match object count"
    );

    assert_eq!(
        idx.offsets_1.len(),
        objects_count,
        "offsets_1 count should match object count"
    );

    let offsets_2_entries = idx
        .offsets_1
        .iter()
        .filter(|entry| (*entry & 0x8000_0000) != 0)
        .count();

    assert_eq!(
        idx.offsets_2.len(),
        offsets_2_entries,
        "offsets_2 count should match count of high-bit offsets in offsets_1"
    );
}
