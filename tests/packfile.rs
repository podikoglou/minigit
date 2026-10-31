mod common;

use std::fs::{self, File};

use include_dir::{Dir, include_dir};
use insta::assert_debug_snapshot;
use similar_asserts::assert_eq;
use tinygit::{
    ende::Encode,
    storage::object::packed::{
        Packfile,
        builder::PackBuilder,
        idx::PackIndex,
        idx_builder::{ObjectCache, PackIndexBuilder},
    },
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
        }
    }
}

#[test]
fn pack_index_encode() {
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
        }
    }
}

#[test]
fn packfile_encode_and_open() {
    let (repo, dir) = unpack_repo(
        REPOS_DIR
            .get_file("repo-1.tar")
            .expect("repo-1.tar fixture should exist")
            .contents(),
    );

    // scan loose objects in the repo
    let objects = repo
        .store
        .objects()
        .expect("should be able to scan repo objects")
        .collect::<Result<Vec<_>, _>>()
        .expect("should get all objects");

    assert!(
        !objects.is_empty(),
        "expected loose objects in repo fixture"
    );

    let mut loose_objects = Vec::new();
    let mut pack_builder = PackBuilder::new();

    for lazy in objects {
        let hash = lazy.hash().expect("should calculate hash");
        let raw = lazy.into_raw().expect("should read raw object");
        pack_builder.insert(hash.clone());
        loose_objects.push((hash, raw));
    }

    // create a packfile out of the scanned objects
    let pack_path = dir.path().join("test.pack");
    let idx_path = dir.path().join("test.idx");

    let mut pack_file = File::create(&pack_path).expect("should create packfile");
    pack_builder
        .flush(&mut pack_file, &repo.store)
        .expect("should flush packfile");
    drop(pack_file);

    // create an index for the packfile
    let index_builder = PackIndexBuilder::new(pack_path.clone(), Some(idx_path.clone()))
        .expect("should create PackIndexBuilder");
    let mut cache = ObjectCache::default();
    let index = index_builder
        .build(&mut cache)
        .expect("should build pack index");
    index.write().expect("should write pack index");

    // open Packfile (struct) using the packfile and its index
    let packfile = Packfile::open(pack_path, idx_path).expect("should open packfile");

    assert_eq!(
        packfile.objects_count as usize,
        loose_objects.len(),
        "packfile object count must match scanned loose objects count"
    );

    // ensure the objects in the packfile are the exact same objects we scanned as loose objects
    for (hash, loose_raw) in &loose_objects {
        let packed_raw = packfile
            .raw_object(hash)
            .expect("should find object in packfile");
        assert_eq!(
            &packed_raw, loose_raw,
            "packed object content and type must match scanned loose object"
        );
    }
}

#[test]
fn packfile_recreate_from_existing_pack() {
    let (repo, _dir) = unpack_repo(
        REPOS_DIR
            .get_file("repo-6.tar")
            .expect("repo-6.tar fixture should exist")
            .contents(),
    );

    // ensure this fixture has only a packfile and no loose objects
    assert_eq!(
        repo.store
            .loose
            .buckets()
            .expect("should list loose buckets")
            .len(),
        0,
        "repo-6 should have no loose objects"
    );
    assert_eq!(
        repo.store.packs().count(),
        1,
        "repo-6 should have exactly one packfile"
    );

    let pack = repo.store.packs().next().expect("should have packfile");
    let pack_path = pack.path().clone();
    let idx_path = pack_path.with_extension("idx");

    // open the packfile and record the objects
    let packfile =
        Packfile::open(pack_path.clone(), idx_path.clone()).expect("should open packfile");
    let mut recorded_objects = Vec::new();
    for hash in &packfile.idx.object_names {
        let raw = packfile.raw_object(hash).expect("should read raw object");
        recorded_objects.push((hash.clone(), raw));
    }
    drop(packfile);

    // delete the packfile (and its index)
    fs::remove_file(&pack_path).expect("should delete packfile");
    fs::remove_file(&idx_path).expect("should delete index");
    let rev_path = pack_path.with_extension("rev");
    if rev_path.exists() {
        let _ = fs::remove_file(rev_path);
    }

    assert!(!pack_path.exists());
    assert!(!idx_path.exists());

    // recreate a packfile and an index
    let mut pack_builder = PackBuilder::new();
    for (hash, _) in &recorded_objects {
        pack_builder.insert(hash.clone());
    }

    let mut new_pack_file = File::create(&pack_path).expect("should create packfile");
    pack_builder
        .flush(&mut new_pack_file, &repo.store)
        .expect("should flush packfile");
    drop(new_pack_file);

    let index_builder = PackIndexBuilder::new(pack_path.clone(), Some(idx_path.clone()))
        .expect("should create PackIndexBuilder");
    let mut cache = ObjectCache::default();
    let index = index_builder
        .build(&mut cache)
        .expect("should build pack index");
    index.write().expect("should write pack index");

    // scan it and ensure the rawobjects are equal
    let new_packfile = Packfile::open(pack_path, idx_path).expect("should open packfile");
    assert_eq!(
        new_packfile.objects_count as usize,
        recorded_objects.len(),
        "recreated packfile object count must match recorded objects"
    );

    for (hash, expected_raw) in &recorded_objects {
        let actual_raw = new_packfile
            .raw_object(hash)
            .expect("should find object in recreated packfile");
        assert_eq!(
            &actual_raw, expected_raw,
            "raw object in recreated packfile must match recorded raw object"
        );
    }
}
