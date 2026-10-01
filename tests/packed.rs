use std::assert_matches;
use std::fs::File;

use memmap2::Mmap;
use minigit::{
    Repo,
    object::Object,
    storage::object::packed::{
        self, BaseObject, CopyInstruction, Delta, Instruction, PackedObject,
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
    let mut slice = &mmap[..];

    let header = packed::header
        .parse_next(&mut slice)
        .expect("should be able to parse packfile header");

    assert_eq!(header.objects, 3, "should have three objects");

    // object 1
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(Object::Commit(_)));
    }

    // object 2
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(Object::Blob(_)));
    }

    // object 3
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(Object::Tree(_)));
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
    let mut slice = &mmap[..];

    let header = packed::header
        .parse_next(&mut slice)
        .expect("should be able to parse packfile header");

    assert_eq!(header.objects, 12, "should have 12 objects");

    // object 1
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(Object::Commit(_)));
    }

    // object 2
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(Object::Commit(_)));
    }

    // object 3
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(Object::Commit(_)));
    }

    // object 4
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(Object::Commit(_)));
    }

    // object 5
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(Object::Blob(_)));
    }

    // object 6
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(Object::Blob(_)));
    }

    // object 7
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(Object::Tree(_)));
    }

    // object 8
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(Object::Tree(_)));
    }

    // object 9 (deltified)
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_eq!(
            object,
            PackedObject::Deltified {
                base: BaseObject::Ofs(264),
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

        assert_matches!(object, PackedObject::Undeltified(Object::Tree(_)));
    }

    // object 11
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(Object::Blob(_)));
    }

    // object 12
    {
        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(Object::Tree(_)));
    }

    assert_eq!(slice.len(), 20, "packfile should end with 20-byte checksum");
}

