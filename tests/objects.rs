use minigit::object::hash::ObjectHash;

mod common;

#[test]
fn objects_iter() {
    let (repo, _dir) = include_repo!("fixtures/repos/repo-1.tar");

    let objects = repo
        .store
        .objects()
        .expect("should be able to list objects");

    assert_eq!(objects.count(), 3, "should have 3 objects");
}

#[test]
fn store_object_retrieval() {
    let (repo, _dir) = include_repo!("fixtures/repos/repo-5.tar");

    // loose object
    let loose_hash: ObjectHash = "ddc67c9b017423980bd3e82879ada0e43a71b8ba"
        .parse()
        .expect("should parse hash");
    let loose_object = repo
        .store
        .object(loose_hash.clone())
        .expect("should find loose object");
    assert_eq!(
        loose_object.hash().expect("should calculate hash"),
        loose_hash
    );

    // packed object
    let packed_hash: ObjectHash = "babbe85f1e25b2b7d2c4321bc9e278bae96b718b"
        .parse()
        .expect("should parse hash");
    let packed_object = repo
        .store
        .object(packed_hash.clone())
        .expect("should find packed object");
    assert_eq!(
        packed_object.hash().expect("should calculate hash"),
        packed_hash
    );
}
