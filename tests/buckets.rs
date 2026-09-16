use minigit::Repo;

mod common;

#[test]
fn bucket_contains_expected_objects() {
    let (repo, _dir) = include_repo!("fixtures/repo-1.tar");

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
