use minigit::storage::Store;

mod common;

#[test]
fn test_objects_iter() {
    let repo = include_repo!("fixtures/repo-1.tar");
    let store = Store::try_new(repo.path().into()).unwrap();

    let count = store.objects().unwrap().count();

    // there is one commit which is one file, thus there are three objects in this repo:
    // - 1 blob
    // - 1 tree
    // - 1 commit
    assert_eq!(count, 3);
}
