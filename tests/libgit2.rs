//! This test suite tests `minigit` against `libgit2` which is used as an oracle and considered to
//! be a correct implementation.
//!
//! NOTE: Because of the way the `git2` crate works, it often happens that while tests may be
//! passing, if you would run this with `-- --nocapture`, it would show errors. This should be kept
//! in mind when running this suite.

use git2::{
    Commit,
    ObjectType::{self as Git2ObjectType},
    Odb, Oid, ReferenceType, Signature,
};
use include_dir::{Dir, include_dir};
use minigit::{
    error::ParserContext,
    identity::{Email, Identity, Name},
    object::{Object, ObjectType as MinigitObjectType, hash::ObjectHash},
    storage::{object::loose::WriteLoose, reference::RefPointer},
};
use std::str::FromStr;

use crate::common::{unpack_git2_repo, unpack_repo};

mod common;

static REPOS_DIR: Dir = include_dir!("tests/fixtures/repos");

fn oid_to_hash(oid: &Oid) -> ObjectHash {
    ObjectHash::from_str(&oid.to_string()).expect("should be able to parse hash")
}

fn signature_to_identity(signature: Signature) -> Identity {
    Identity {
        name: Name::try_new(signature.name().unwrap().into()).unwrap(),
        email: Email::new(signature.email().unwrap().into()),
    }
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

fn collect_oids(odb: &Odb<'_>) -> Vec<Oid> {
    let mut oids = Vec::new();

    odb.foreach(|oid| {
        oids.push(oid.to_owned());
        true
    })
    .expect("should be able to iterate over OIDs");

    oids
}

#[test]
fn raw_objects_match() {
    for repo_dir in REPOS_DIR.files() {
        let (minigit_repo, _dir) = unpack_repo(repo_dir.contents());
        let (oracle_repo, _dir) = unpack_git2_repo(repo_dir.contents());

        let oracle_odb = oracle_repo.odb().expect("should be able to open odb");
        let minigit_store = minigit_repo.store;

        for oid in collect_oids(&oracle_odb) {
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
        }
    }
}

#[test]
fn commits_match() {
    for repo_dir in REPOS_DIR.files() {
        let (minigit_repo, _dir) = unpack_repo(repo_dir.contents());
        let (oracle_repo, _dir) = unpack_git2_repo(repo_dir.contents());

        let oracle_odb = oracle_repo.odb().expect("should be able to open odb");
        let minigit_store = minigit_repo.store;

        for oid in collect_oids(&oracle_odb) {
            // minigit object
            let hash = oid_to_hash(&oid);

            let minigit_raw_obj = minigit_store
                .raw_object(hash)
                .expect("minigit should load the object");

            let minigit_obj = minigit_raw_obj
                .into_object(ParserContext::None)
                .expect("should be able to parse the bject");

            let Object::Commit(minigit_commit) = minigit_obj else {
                continue;
            };

            // oracle object
            let oracle_odb_obj = oracle_odb
                .read(oid)
                .expect("libgit2 should load the object");

            let oracle_obj = oracle_repo
                .find_object(oid, Some(oracle_odb_obj.kind()))
                .expect("libgit2 should be able to find the object");

            let Some(oracle_commit) = oracle_obj.as_commit() else {
                continue;
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
                minigit_commit.author.0,
                signature_to_identity(oracle_commit.author()),
                "authors match"
            );

            assert_eq!(
                minigit_commit.committer.0,
                signature_to_identity(oracle_commit.committer()),
                "commiters match"
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
        }
    }
}

#[test]
fn trees_match() {
    for repo_dir in REPOS_DIR.files() {
        let (minigit_repo, _dir) = unpack_repo(repo_dir.contents());
        let (oracle_repo, _dir) = unpack_git2_repo(repo_dir.contents());

        let oracle_odb = oracle_repo.odb().expect("should be able to open odb");
        let minigit_store = minigit_repo.store;

        for oid in collect_oids(&oracle_odb) {
            // minigit object
            let hash = oid_to_hash(&oid);

            let minigit_raw_obj = minigit_store
                .raw_object(hash)
                .expect("minigit should load the object");

            let minigit_obj = minigit_raw_obj
                .into_object(ParserContext::None)
                .expect("should be able to parse the bject");

            let Object::Tree(minigit_tree) = minigit_obj else {
                continue;
            };

            // oracle object
            let oracle_odb_obj = oracle_odb
                .read(oid)
                .expect("libgit2 should load the object");

            let oracle_obj = oracle_repo
                .find_object(oid, Some(oracle_odb_obj.kind()))
                .expect("libgit2 should be able to find the object");

            let Some(oracle_tree) = oracle_obj.as_tree() else {
                continue;
            };

            assert_eq!(
                minigit_tree.entries.len(),
                oracle_tree.len(),
                "tree lengths match"
            );

            for (oracle_entry, minigit_entry) in oracle_tree.iter().zip(minigit_tree.entries.iter())
            {
                assert_eq!(
                    minigit_entry.0.as_str(),
                    oracle_entry.name().unwrap(),
                    "file names match"
                );

                assert_eq!(
                    minigit_entry.0.mode as i32,
                    oracle_entry.filemode(),
                    "filemode mathces"
                );

                assert_eq!(
                    minigit_entry.1.object,
                    oid_to_hash(&oracle_entry.id()),
                    "entry object hashes match"
                );
            }
        }
    }
}

#[test]
fn blobs_match() {
    for repo_dir in REPOS_DIR.files() {
        let (minigit_repo, _dir) = unpack_repo(repo_dir.contents());
        let (oracle_repo, _dir) = unpack_git2_repo(repo_dir.contents());

        let oracle_odb = oracle_repo.odb().expect("should be able to open odb");
        let minigit_store = minigit_repo.store;

        for oid in collect_oids(&oracle_odb) {
            // minigit object
            let hash = oid_to_hash(&oid);

            let minigit_raw_obj = minigit_store
                .raw_object(hash)
                .expect("minigit should load the object");

            let minigit_obj = minigit_raw_obj
                .into_object(ParserContext::None)
                .expect("should be able to parse the bject");

            let Object::Blob(minigit_blob) = minigit_obj else {
                continue;
            };

            // oracle object
            let oracle_odb_obj = oracle_odb
                .read(oid)
                .expect("libgit2 should load the object");

            let oracle_obj = oracle_repo
                .find_object(oid, Some(oracle_odb_obj.kind()))
                .expect("libgit2 should be able to find the object");

            let Some(oracle_blob) = oracle_obj.as_blob() else {
                continue;
            };

            assert_eq!(minigit_blob.0.len(), oracle_blob.size(), "sizes match");
            assert_eq!(minigit_blob.0, oracle_blob.content(), "contents match");
        }
    }
}

#[test]
fn tags_match() {
    for repo_dir in REPOS_DIR.files() {
        let (minigit_repo, _dir) = unpack_repo(repo_dir.contents());
        let (oracle_repo, _dir) = unpack_git2_repo(repo_dir.contents());

        let oracle_odb = oracle_repo.odb().expect("should be able to open odb");
        let minigit_store = minigit_repo.store;

        for oid in collect_oids(&oracle_odb) {
            // minigit object
            let hash = oid_to_hash(&oid);

            let minigit_raw_obj = minigit_store
                .raw_object(hash)
                .expect("minigit should load the object");

            let minigit_obj = minigit_raw_obj
                .into_object(ParserContext::None)
                .expect("should be able to parse the bject");

            let Object::Tag(minigit_tag) = minigit_obj else {
                continue;
            };

            // oracle object
            let oracle_odb_obj = oracle_odb
                .read(oid)
                .expect("libgit2 should load the object");

            let oracle_obj = oracle_repo
                .find_object(oid, Some(oracle_odb_obj.kind()))
                .expect("libgit2 should be able to find the object");

            let Some(oracle_tag) = oracle_obj.as_tag() else {
                continue;
            };

            assert_eq!(
                minigit_tag.name.as_str(),
                oracle_tag.name().unwrap(),
                "names match"
            );

            assert_eq!(
                minigit_tag.description.as_str(),
                oracle_tag.message().unwrap().unwrap_or_default(),
                "descriptions match"
            );

            assert_eq!(
                minigit_tag.tagger.map(|x| x.0),
                oracle_tag.tagger().map(signature_to_identity),
                "taggers match"
            );
        }
    }
}

#[test]
fn commits_reserialization_match() {
    for repo_dir in REPOS_DIR.files() {
        let (minigit_repo, _dir) = unpack_repo(repo_dir.contents());
        let (oracle_repo, _dir) = unpack_git2_repo(repo_dir.contents());

        let oracle_odb = oracle_repo.odb().expect("should be able to open odb");
        let minigit_store = minigit_repo.store;

        for oid in collect_oids(&oracle_odb) {
            let hash = oid_to_hash(&oid);

            let minigit_raw_obj = minigit_store
                .raw_object(hash)
                .expect("minigit should load the object");

            let minigit_obj = minigit_raw_obj
                .into_object(ParserContext::None)
                .expect("minigit should parse the object");

            let Object::Commit(minigit_commit) = minigit_obj else {
                continue;
            };

            let oracle_odb_obj = oracle_odb
                .read(oid)
                .expect("libgit2 should load the object");

            let oracle_obj = oracle_repo
                .find_object(oid, Some(oracle_odb_obj.kind()))
                .expect("libgit2 should find the object");

            let Some(oracle_commit) = oracle_obj.as_commit() else {
                continue;
            };

            let parents: Vec<_> = oracle_commit.parents().collect();
            let parent_refs: Vec<&Commit> = parents.iter().collect();
            let tree = oracle_commit.tree().expect("commit tree");
            let libgit2_buf = oracle_repo
                .commit_create_buffer(
                    &oracle_commit.author(),
                    &oracle_commit.committer(),
                    oracle_commit.message_raw().unwrap(),
                    &tree,
                    &parent_refs,
                )
                .expect("libgit2 should serialize commit");

            let mut minigit_bytes = Vec::new();
            minigit_commit
                .write_loose(&mut minigit_bytes)
                .expect("minigit should serialize commit");

            assert_eq!(
                minigit_bytes.as_slice(),
                &libgit2_buf[..],
                "serialized commit bytes match"
            );
        }
    }
}

#[test]
fn trees_reserialization_match() {
    for repo_dir in REPOS_DIR.files() {
        let (minigit_repo, _dir) = unpack_repo(repo_dir.contents());
        let (oracle_repo, _dir) = unpack_git2_repo(repo_dir.contents());

        let oracle_odb = oracle_repo.odb().expect("should be able to open odb");
        let minigit_store = minigit_repo.store;

        for oid in collect_oids(&oracle_odb) {
            let hash = oid_to_hash(&oid);

            let minigit_raw_obj = minigit_store
                .raw_object(hash)
                .expect("minigit should load the object");

            let minigit_obj = minigit_raw_obj
                .into_object(ParserContext::None)
                .expect("minigit should parse the object");

            let Object::Tree(minigit_tree) = minigit_obj else {
                continue;
            };

            let oracle_odb_obj = oracle_odb
                .read(oid)
                .expect("libgit2 should load the object");

            let oracle_obj = oracle_repo
                .find_object(oid, Some(oracle_odb_obj.kind()))
                .expect("libgit2 should find the object");

            let Some(oracle_tree) = oracle_obj.as_tree() else {
                continue;
            };

            let builder = oracle_repo
                .treebuilder(Some(oracle_tree))
                .expect("treebuilder");
            let new_tree_oid = builder.write().expect("treebuilder write");
            let libgit2_tree_obj = oracle_odb.read(new_tree_oid).expect("read tree");

            let mut minigit_bytes = Vec::new();
            minigit_tree
                .write_loose(&mut minigit_bytes)
                .expect("minigit should serialize tree");

            assert_eq!(
                minigit_bytes,
                libgit2_tree_obj.data(),
                "serialized tree bytes match"
            );
        }
    }
}

#[test]
fn blobs_reserialization_match() {
    for repo_dir in REPOS_DIR.files() {
        let (minigit_repo, _dir) = unpack_repo(repo_dir.contents());
        let (oracle_repo, _dir) = unpack_git2_repo(repo_dir.contents());

        let oracle_odb = oracle_repo.odb().expect("should be able to open odb");
        let minigit_store = minigit_repo.store;

        for oid in collect_oids(&oracle_odb) {
            let hash = oid_to_hash(&oid);

            let minigit_raw_obj = minigit_store
                .raw_object(hash)
                .expect("minigit should load the object");

            let minigit_obj = minigit_raw_obj
                .into_object(ParserContext::None)
                .expect("minigit should parse the object");

            let Object::Blob(minigit_blob) = minigit_obj else {
                continue;
            };

            let oracle_odb_obj = oracle_odb
                .read(oid)
                .expect("libgit2 should load the object");

            let oracle_obj = oracle_repo
                .find_object(oid, Some(oracle_odb_obj.kind()))
                .expect("libgit2 should find the object");

            let Some(oracle_blob) = oracle_obj.as_blob() else {
                continue;
            };

            let new_blob_oid = oracle_repo.blob(oracle_blob.content()).expect("write blob");
            let libgit2_blob_obj = oracle_odb.read(new_blob_oid).expect("read blob");

            let mut minigit_bytes = Vec::new();
            minigit_blob
                .write_loose(&mut minigit_bytes)
                .expect("minigit should serialize blob");

            assert_eq!(
                minigit_bytes,
                libgit2_blob_obj.data(),
                "serialized blob bytes match"
            );
        }
    }
}

#[test]
fn tags_reserialization_match() {
    for repo_dir in REPOS_DIR.files() {
        let (minigit_repo, _dir) = unpack_repo(repo_dir.contents());
        let (oracle_repo, _dir) = unpack_git2_repo(repo_dir.contents());

        let oracle_odb = oracle_repo.odb().expect("should be able to open odb");
        let minigit_store = minigit_repo.store;

        for oid in collect_oids(&oracle_odb) {
            let hash = oid_to_hash(&oid);

            let minigit_raw_obj = minigit_store
                .raw_object(hash)
                .expect("minigit should load the object");

            let minigit_obj = minigit_raw_obj
                .into_object(ParserContext::None)
                .expect("minigit should parse the object");

            let Object::Tag(minigit_tag) = minigit_obj else {
                continue;
            };

            let oracle_odb_obj = oracle_odb
                .read(oid)
                .expect("libgit2 should load the object");

            let oracle_obj = oracle_repo
                .find_object(oid, Some(oracle_odb_obj.kind()))
                .expect("libgit2 should find the object");

            let Some(oracle_tag) = oracle_obj.as_tag() else {
                continue;
            };

            let target = oracle_tag.target().expect("tag target");
            let tagger = oracle_tag.tagger().expect("tagger");
            let new_tag_oid = oracle_repo
                .tag_annotation_create(
                    oracle_tag.name().unwrap(),
                    &target,
                    &tagger,
                    oracle_tag.message().unwrap().unwrap_or_default(),
                )
                .expect("tag_annotation_create");
            let libgit2_tag_obj = oracle_odb.read(new_tag_oid).expect("read tag");

            let mut minigit_bytes = Vec::new();
            minigit_tag
                .write_loose(&mut minigit_bytes)
                .expect("minigit should serialize tag");

            assert_eq!(
                minigit_bytes,
                libgit2_tag_obj.data(),
                "serialized tag bytes match"
            );
        }
    }
}

#[test]
fn refs_match() {
    for repo_dir in REPOS_DIR.files() {
        let (minigit_repo, _dir) = unpack_repo(repo_dir.contents());
        let (oracle_repo, _dir) = unpack_git2_repo(repo_dir.contents());

        let minigit_store = minigit_repo.store;
        let oracle_refs = oracle_repo
            .references()
            .expect("libgit2 should list references");

        for oracle_ref in oracle_refs {
            let oracle_ref = oracle_ref.expect("valid reference");
            let ref_name = oracle_ref.name().expect("utf-8 reference name");

            let minigit_ref = minigit_store
                .reference(ref_name.to_string())
                .expect("minigit should load the reference");

            assert_eq!(minigit_ref.name, ref_name, "reference names match");

            match oracle_ref.kind() {
                Some(ReferenceType::Direct) => {
                    let oracle_target = oracle_ref.target().expect("direct ref should have target");
                    assert_eq!(
                        minigit_ref.pointer,
                        RefPointer::Hash(oid_to_hash(&oracle_target)),
                        "direct ref targets match"
                    );
                }
                Some(ReferenceType::Symbolic) => {
                    let oracle_target = oracle_ref
                        .symbolic_target()
                        .expect("valid symbolic target")
                        .expect("symbolic ref should have target");
                    assert_eq!(
                        minigit_ref.pointer,
                        RefPointer::Symbolic(oracle_target.to_string()),
                        "symbolic ref targets match"
                    );
                }
                None => unreachable!(),
            }

            let oracle_resolved = oracle_ref
                .resolve()
                .expect("libgit2 should resolve reference");
            let oracle_resolved_oid = oracle_resolved
                .target()
                .expect("resolved ref should have target");

            let minigit_resolved_hash = minigit_store
                .resolve_reference(&minigit_ref.pointer)
                .expect("minigit should resolve reference");

            assert_eq!(
                minigit_resolved_hash,
                oid_to_hash(&oracle_resolved_oid),
                "resolved reference hashes match"
            );
        }
    }
}

#[test]
fn head_matches() {
    for repo_dir in REPOS_DIR.files() {
        let (minigit_repo, _dir) = unpack_repo(repo_dir.contents());
        let (oracle_repo, _dir) = unpack_git2_repo(repo_dir.contents());

        let minigit_store = minigit_repo.store;

        let (minigit_commit, minigit_head_hash) =
            minigit_store.head().expect("minigit should resolve HEAD");

        let oracle_head = oracle_repo.head().expect("libgit2 should find HEAD");
        let oracle_commit = oracle_head
            .peel_to_commit()
            .expect("libgit2 should peel HEAD to commit");

        assert_eq!(
            minigit_head_hash,
            oid_to_hash(&oracle_commit.id()),
            "HEAD commit hashes match"
        );

        assert_eq!(
            minigit_commit.tree,
            oid_to_hash(&oracle_commit.tree_id()),
            "HEAD commit tree matches"
        );

        assert_eq!(
            minigit_commit.parents,
            oracle_commit
                .parent_ids()
                .map(|oid| oid_to_hash(&oid))
                .collect::<Vec<_>>(),
            "HEAD commit parents match"
        );

        assert_eq!(
            minigit_commit.author.0,
            signature_to_identity(oracle_commit.author()),
            "HEAD commit authors match"
        );

        assert_eq!(
            minigit_commit.committer.0,
            signature_to_identity(oracle_commit.committer()),
            "HEAD commit committers match"
        );

        assert_eq!(
            minigit_commit.author.1.timestamp(),
            oracle_commit.time().seconds(),
            "HEAD commit times match"
        );

        assert_eq!(
            minigit_commit.description,
            oracle_commit.message().unwrap(),
            "HEAD commit messages match"
        );
    }
}
