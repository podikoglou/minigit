mod common;

use std::fs;

use include_dir::{include_dir, Dir};
use insta::assert_debug_snapshot;
use minigit::storage::object::{
    loose::Encode,
    packed::{
        idx::PackIndex,
        idx_builder::{ObjectCache, PackIndexBuilder},
    },
};
use similar_asserts::assert_eq;

use crate::common::unpack_repo;

static REPOS_DIR: Dir = include_dir!("tests/fixtures/repos");

#[test]
fn store_packs_listing() {
    let (repo, _dir) = include_repo!("fixtures/repos/repo-5.tar");

    for pack in repo.store.packs() {
        assert_debug_snapshot!((&pack.name, &pack.objects_count));
    }
}

#[test]
fn pack_index_builder() {
    let mut tested_packs = 0;

    for repo_file in REPOS_DIR.files() {
        let (repo, _dir) = unpack_repo(repo_file.contents());

        for pack in repo.store.packs() {
            let (pack_path, _) = &pack.pack;
            let idx_path = pack_path.with_extension("idx");
            let expected_idx =
                PackIndex::open(&idx_path).expect("should be able to parse packfile index");

            let builder = PackIndexBuilder::new(pack_path.clone(), None)
                .expect("should create PackIndexBuilder");
            let mut cache = ObjectCache::default();
            let built_idx = builder
                .build(&mut cache)
                .expect("should be able to build pack index");

            assert_eq!(
                built_idx.objects_count, expected_idx.objects_count,
                "objects_count mismatch"
            );
            assert_eq!(
                built_idx.fanout_table, expected_idx.fanout_table,
                "fanout_table mismatch"
            );
            assert_eq!(
                built_idx.object_names, expected_idx.object_names,
                "object_names mismatch"
            );
            assert_eq!(
                built_idx.crc_entries, expected_idx.crc_entries,
                "crc_entries mismatch"
            );
            assert_eq!(
                built_idx.offsets_1, expected_idx.offsets_1,
                "offsets_1 mismatch"
            );
            assert_eq!(
                built_idx.offsets_2, expected_idx.offsets_2,
                "offsets_2 mismatch"
            );
            assert_eq!(
                built_idx.pack_checksum, expected_idx.pack_checksum,
                "pack_checksum mismatch"
            );

            // this is hard to check, becauase it's not actually stored when building, it's only
            // computed upon encoding

            // assert_eq!(
            //     built_idx.idx_checksum, expected_idx.idx_checksum,
            //     "idx_checksum mismatch"
            // );

            tested_packs += 1;
        }
    }

    assert!(
        tested_packs > 0,
        "expected to find packfiles in repo fixtures"
    );
}

#[test]
fn pack_index_encode() {
    let mut tested_packs = 0;

    for repo_file in REPOS_DIR.files() {
        let (repo, _dir) = unpack_repo(repo_file.contents());

        for pack in repo.store.packs() {
            let (pack_path, _) = &pack.pack;
            let idx_path = pack_path.with_extension("idx");
            let old_idx = fs::read(&idx_path).expect("should be able to read packfile index");

            let builder = PackIndexBuilder::new(pack_path.clone(), None)
                .expect("should create PackIndexBuilder");
            let mut cache = ObjectCache::default();
            let built_idx = builder
                .build(&mut cache)
                .expect("should be able to build pack index");

            let mut new_idx = Vec::new();
            built_idx
                .encode(&mut new_idx)
                .expect("should be able to encode pack index");

            assert_eq!(new_idx, old_idx);
            tested_packs += 1;
        }
    }

    assert!(
        tested_packs > 0,
        "expected to find packfiles in repo fixtures"
    );
}
