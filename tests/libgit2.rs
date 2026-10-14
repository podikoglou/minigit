//! This test suite tests `minigit` against `libgit2` which is used as an oracle and considered to
//! be a correct implementation.

use std::str::FromStr;

use git2::Oid;
use minigit::object::hash::ObjectHash;

mod common;

fn oid_to_hash(oid: &Oid) -> ObjectHash {
    ObjectHash::from_str(&oid.to_string()).expect("should be able to parse hash")
}

#[test]
fn repo_5() {
    let (minigit_repo, _dir) = include_repo!("fixtures/repos/repo-5.tar");
    let (oracle_repo, _dir) = include_repo_git2!("fixtures/repos/repo-5.tar");

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

            true
        })
        .expect("foreach should work");
}
