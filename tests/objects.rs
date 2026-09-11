use minigit::Repo;

mod common;

#[test]
fn test_objects_iter() {
    let path = include_repo!("fixtures/repo-1.tar");
    let repo = Repo::open(path.path().into()).unwrap();

    let count = repo.store.objects().unwrap().count();

    // there is one commit which contains one file, thus there are three objects in this repo:
    // - 1 blob
    // - 1 tree
    // - 1 commit
    assert_eq!(count, 3);
}
