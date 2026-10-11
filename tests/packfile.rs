mod common;

use minigit::Repo;

#[test]
fn store_packs_listing() {
    let (repo, _dir) = include_repo!("fixtures/repo-5.tar");

    let packs = repo.store.packs().expect("should be able to list packs");

    assert_eq!(packs.len(), 1);
}
