use std::fs::File;

use memmap2::Mmap;
use minigit::{
    Repo,
    object::Object,
    storage::object::packed::{self, PackedObject, PackedObjectType},
};
use winnow::Parser;

use std::assert_matches;

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
        let object_header = packed::object_header
            .parse_next(&mut slice)
            .expect("should be able to parse object header");

        assert_eq!(object_header.r#type, PackedObjectType::Commit);

        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(Object::Commit(_)));
    }

    // object 2
    {
        let object_header = packed::object_header
            .parse_next(&mut slice)
            .expect("should be able to parse object header");

        assert_eq!(object_header.r#type, PackedObjectType::Blob);

        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(Object::Blob(_)));
    }

    // object 3
    {
        let object_header = packed::object_header
            .parse_next(&mut slice)
            .expect("should be able to parse object header");

        assert_eq!(object_header.r#type, PackedObjectType::Tree);

        let object = packed::object
            .parse_next(&mut slice)
            .expect("should be able to parse object");

        assert_matches!(object, PackedObject::Undeltified(Object::Tree(_)));
    }
}
