//! This test suite tests `minigit` against `libgit2` which is used as an oracle and considered to
//! be a correct implementation.

use bstr::ByteSlice;
use git2::{
    Commit,
    ObjectType::{self as Git2ObjectType},
    Oid,
};
use include_dir::{Dir, include_dir};
use minigit::{
    error::ParserContext,
    object::{Object, ObjectType as MinigitObjectType, hash::ObjectHash},
};
use std::str::FromStr;

use crate::common::{unpack_git2_repo, unpack_repo};

mod common;

static REPOS_DIR: Dir = include_dir!("tests/fixtures/repos");

fn oid_to_hash(oid: &Oid) -> ObjectHash {
    ObjectHash::from_str(&oid.to_string()).expect("should be able to parse hash")
}

fn git2_object_type_to_object_type(obj_type: Git2ObjectType) -> Option<MinigitObjectType> {
    match obj_type {
        Git2ObjectType::Any => None,
        Git2ObjectType::Commit => Some(MinigitObjectType::Commit),
        Git2ObjectType::Tree => Some(MinigitObjectType::Tree),
        Git2ObjectType::Blob => Some(MinigitObjectType::Blob),
        Git2ObjectType::Tag => Some(MinigitObjectType::Tag),
    }
}

#[test]
fn raw_objects_match() {
    for repo_dir in REPOS_DIR.files() {
        let (minigit_repo, _dir) = unpack_repo(repo_dir.contents());
        let (oracle_repo, _dir) = unpack_git2_repo(repo_dir.contents());

        let oracle_odb = oracle_repo.odb().expect("should be able to open odb");
        let minigit_store = minigit_repo.store;

        oracle_odb
            .foreach(|oid| {
                let oid = oid.to_owned();

                let hash = oid_to_hash(&oid);

                let minigit_raw_obj = minigit_store
                    .raw_object(hash)
                    .expect("minigit should load the object");

                let oracle_obj = oracle_odb
                    .read(oid)
                    .expect("libgit2 should load the object");

                assert_eq!(
                    minigit_raw_obj.bytes.len(),
                    oracle_obj.data().len(),
                    "object lengths should be equal"
                );

                assert_eq!(
                    minigit_raw_obj.bytes,
                    oracle_obj.data(),
                    "raw object data should be equal"
                );

                let oracle_obj_type = oracle_obj.kind();
                let oracle_obj_type = git2_object_type_to_object_type(oracle_obj_type);

                assert_eq!(
                    Some(minigit_raw_obj.r#type),
                    oracle_obj_type,
                    "object types should match"
                );

                true
            })
            .expect("foreach should work");
    }
}

#[test]
fn commits_match() {
    for repo_dir in REPOS_DIR.files() {
        let (minigit_repo, _dir) = unpack_repo(repo_dir.contents());
        let (oracle_repo, _dir) = unpack_git2_repo(repo_dir.contents());

        let oracle_odb = oracle_repo.odb().expect("should be able to open odb");
        let minigit_store = minigit_repo.store;

        oracle_odb
            .foreach(|oid| {
                let oid = oid.to_owned();

                // minigit object
                let hash = oid_to_hash(&oid);

                let minigit_raw_obj = minigit_store
                    .raw_object(hash)
                    .expect("minigit should load the object");

                let minigit_obj = minigit_raw_obj
                    .into_object(ParserContext::None)
                    .expect("should be able to parse the bject");

                let Object::Commit(minigit_commit) = minigit_obj else {
                    return true;
                };

                // oracle object
                let oracle_odb_obj = oracle_odb
                    .read(oid)
                    .expect("libgit2 should load the object");

                let oracle_obj = oracle_repo
                    .find_object(oid, Some(oracle_odb_obj.kind()))
                    .expect("libgit2 should be able to find the object");

                let Some(oracle_commit) = oracle_obj.as_commit() else {
                    return true;
                };

                assert_eq!(
                    minigit_commit.tree,
                    oid_to_hash(&oracle_commit.tree_id()),
                    "tree matches"
                );

                assert_eq!(
                    minigit_commit.parents,
                    oracle_commit
                        .parent_ids()
                        .map(|oid| oid_to_hash(&oid))
                        .collect::<Vec<_>>(),
                    "parents match"
                );

                assert_eq!(
                    minigit_commit.author.0.name.to_str().unwrap(),
                    oracle_commit.author().name().unwrap(),
                    "author names match"
                );

                assert_eq!(
                    minigit_commit.author.0.email.to_str().unwrap(),
                    oracle_commit.author().email().unwrap(),
                    "author emails match"
                );

                assert_eq!(
                    minigit_commit.committer.0.name.to_str().unwrap(),
                    oracle_commit.committer().name().unwrap(),
                    "committer names match"
                );

                assert_eq!(
                    minigit_commit.committer.0.email.to_str().unwrap(),
                    oracle_commit.committer().email().unwrap(),
                    "committer emails match"
                );

                assert_eq!(
                    minigit_commit.author.1.timestamp(),
                    oracle_commit.time().seconds(),
                    "commit times match"
                );

                assert_eq!(
                    minigit_commit.description,
                    oracle_commit.message().unwrap(),
                    "message matches"
                );

                // TODO: test for headers

                true
            })
            .expect("foreach should work");
    }
}

#[test]
fn trees_match() {
    for repo_dir in REPOS_DIR.files() {
        let (minigit_repo, _dir) = unpack_repo(repo_dir.contents());
        let (oracle_repo, _dir) = unpack_git2_repo(repo_dir.contents());

        let oracle_odb = oracle_repo.odb().expect("should be able to open odb");
        let minigit_store = minigit_repo.store;

        oracle_odb
            .foreach(|oid| {
                let oid = oid.to_owned();

                // minigit object
                let hash = oid_to_hash(&oid);

                let minigit_raw_obj = minigit_store
                    .raw_object(hash)
                    .expect("minigit should load the object");

                let minigit_obj = minigit_raw_obj
                    .into_object(ParserContext::None)
                    .expect("should be able to parse the bject");

                let Object::Tree(minigit_tree) = minigit_obj else {
                    return true;
                };

                // oracle object
                let oracle_odb_obj = oracle_odb
                    .read(oid)
                    .expect("libgit2 should load the object");

                let oracle_obj = oracle_repo
                    .find_object(oid, Some(oracle_odb_obj.kind()))
                    .expect("libgit2 should be able to find the object");

                let Some(oracle_tree) = oracle_obj.as_tree() else {
                    return true;
                };

                assert_eq!(
                    minigit_tree.entries.len(),
                    oracle_tree.len(),
                    "tree lengths match"
                );

                for (oracle_entry, minigit_entry) in
                    oracle_tree.iter().zip(minigit_tree.entries.iter())
                {
                    assert_eq!(
                        minigit_entry.0.as_str(),
                        oracle_entry.name().unwrap(),
                        "file names match"
                    );

                    assert_eq!(
                        minigit_entry.1.mode as i32,
                        oracle_entry.filemode(),
                        "filemode mathces"
                    );

                    assert_eq!(
                        minigit_entry.1.object,
                        oid_to_hash(&oracle_entry.id()),
                        "entry object hashes match"
                    );
                }

                true
            })
            .expect("foreach should work");
    }
}

#[test]
fn blobs_match() {
    for repo_dir in REPOS_DIR.files() {
        let (minigit_repo, _dir) = unpack_repo(repo_dir.contents());
        let (oracle_repo, _dir) = unpack_git2_repo(repo_dir.contents());

        let oracle_odb = oracle_repo.odb().expect("should be able to open odb");
        let minigit_store = minigit_repo.store;

        oracle_odb
            .foreach(|oid| {
                let oid = oid.to_owned();

                // minigit object
                let hash = oid_to_hash(&oid);

                let minigit_raw_obj = minigit_store
                    .raw_object(hash)
                    .expect("minigit should load the object");

                let minigit_obj = minigit_raw_obj
                    .into_object(ParserContext::None)
                    .expect("should be able to parse the bject");

                let Object::Blob(minigit_blob) = minigit_obj else {
                    return true;
                };

                // oracle object
                let oracle_odb_obj = oracle_odb
                    .read(oid)
                    .expect("libgit2 should load the object");

                let oracle_obj = oracle_repo
                    .find_object(oid, Some(oracle_odb_obj.kind()))
                    .expect("libgit2 should be able to find the object");

                let Some(oracle_blob) = oracle_obj.as_blob() else {
                    return true;
                };

                assert_eq!(minigit_blob.0.len(), oracle_blob.size(), "sizes match");
                assert_eq!(minigit_blob.0, oracle_blob.content(), "contents match");

                true
            })
            .expect("foreach should work");
    }
}
