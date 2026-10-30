mod common;

use include_dir::{Dir, include_dir};
use insta::assert_debug_snapshot;
use similar_asserts::assert_eq;
use minigit::storage::object::packed::{
    idx::PackIndex,
    idx_builder::{ObjectCache, PackIndexBuilder},
};

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
    let mut packfiles = Vec::new();
    let mut repo_dirs = Vec::new();

    for repo_file in REPOS_DIR.files() {
        let (repo, dir) = unpack_repo(repo_file.contents());

        for pack in repo.store.packs() {
            let (pack_path, _) = &pack.pack;
            let idx_path = pack_path.with_extension("idx");
            let expected_idx =
                PackIndex::open(&idx_path).expect("should be able to parse packfile index");

            packfiles.push((pack_path.clone(), expected_idx));
        }

        repo_dirs.push(dir);
    }

    assert!(
        !packfiles.is_empty(),
        "expected to find packfiles in repo fixtures"
    );

    for (pack_path, expected_idx) in packfiles {
        let builder =
            PackIndexBuilder::new(pack_path, None).expect("should create PackIndexBuilder");
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
        assert_eq!(
            built_idx.idx_checksum, expected_idx.idx_checksum,
            "idx_checksum mismatch"
        );
    }
}
