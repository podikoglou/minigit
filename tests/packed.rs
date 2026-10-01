use std::fs::File;

use memmap2::Mmap;
use minigit::{Repo, storage::object::packed};
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

    assert_eq!(header.objects, 3);
}
