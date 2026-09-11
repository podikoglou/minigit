use minigit::storage::Store;

mod common;

#[test]
fn test_buckets_iter() {
    let repo = include_repo!("fixtures/repo-1.tar");
    let store = Store::try_new(repo.path().into()).unwrap();

    // there is one commit which contains one file, thus there are three objects in this repo. in
    // repo-1 they happen to start with different prefixes, thus there are three buckets.
    assert!(store.buckets().is_ok_and(|buckets| buckets.len() == 3));
}
