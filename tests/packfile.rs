mod common;

use fallible_iterator::FallibleIterator;
use insta::assert_debug_snapshot;

#[test]
fn store_packs_listing() {
    let (repo, _dir) = include_repo!("fixtures/repos/repo-5.tar");

    let packs = repo.store.packs().expect("should be able to list packs");

    assert_eq!(packs.len(), 1);

    let pack = packs.first().unwrap();
    assert_eq!(
        pack.objects()
            .count()
            .expect("should be able to iterate through all objects"),
        3
    );
}

#[test]
fn pack_name() {
    let (repo, _dir) = include_repo!("fixtures/repos/repo-5.tar");

    let packs = repo.store.packs().expect("should be able to list packs");

    assert_eq!(packs.len(), 1);

    let pack = packs.first().unwrap();

    assert_debug_snapshot!(&pack.name);
}
