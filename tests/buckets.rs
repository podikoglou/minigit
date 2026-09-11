use minigit::Repo;

mod common;

#[test]
fn test_buckets_iter() {
    let path = include_repo!("fixtures/repo-1.tar");
    let repo = Repo::open(path.path()).unwrap();

    // there is one commit which contains one file, thus there are three objects in this repo. in
    // repo-1 they happen to start with different prefixes, thus there are three buckets.
    assert!(repo.store.buckets().is_ok_and(|buckets| buckets.len() == 3));
}
