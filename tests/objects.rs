use minigit::Repo;

mod common;

#[test]
fn test_objects_iter() {
    let path = include_repo!("fixtures/repo-1.tar");
    let repo = Repo::open(path.path()).expect("should be able to open repo");

    let objects = repo
        .store
        .objects()
        .expect("should be able to list objects");

    assert_eq!(objects.count(), 3, "should have 3 objects");
}
