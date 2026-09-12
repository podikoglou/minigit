use minigit::Repo;

mod common;

#[test]
fn test_bucket_contains_expected_objects() {
    let path = include_repo!("fixtures/repo-1.tar");
    let repo = Repo::open(path.path()).expect("should be able to open repo");

    let buckets = repo
        .store
        .buckets()
        .expect("should be able to list buckets");

    assert_eq!(buckets.len(), 3, "should have three buckets");

    for prefix in [0xbe, 0x7b, 0xfa] {
        assert!(
            buckets.iter().find(|e| e.prefix == prefix.into()).is_some(),
            "should have bucket with expected prefix"
        );
    }
}
