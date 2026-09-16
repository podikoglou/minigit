use minigit::Repo;

mod common;

#[test]
fn objects_iter() {
    let (repo, _dir) = include_repo!("fixtures/repo-1.tar");

    let objects = repo
        .store
        .objects()
        .expect("should be able to list objects");

    assert_eq!(objects.count(), 3, "should have 3 objects");
}
