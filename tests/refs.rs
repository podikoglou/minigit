use insta::assert_debug_snapshot;

mod common;

// #[test]
// fn lazy_refs_iter_repo_3() {
//     let (repo, _dir) = include_repo!("fixtures/repos/repo-3.tar");
//     let refs: Vec<Result<LazyRef, TinygitError>> = repo.store.lazy_refs().collect();
//     let refs: Vec<_> = refs
//         .into_iter()
//         .map(|x| x.map(|x| (x.name, x.pointer)))
//         .collect();
//
//     insta::assert_debug_snapshot!(refs);
// }
//
// #[test]
// fn lazy_refs_iter_repo_4() {
//     let (repo, _dir) = include_repo!("fixtures/repos/repo-4.tar");
//     let refs: Vec<Result<LazyRef, TinygitError>> = repo.store.lazy_refs().collect();
//     let refs: Vec<_> = refs
//         .into_iter()
//         .map(|x| x.map(|x| (x.name, x.pointer)))
//         .collect();
//
//     insta::assert_debug_snapshot!(refs);
// }

#[test]
fn ref_db_lookup() {
    let (repo, _dir) = include_repo!("fixtures/repos/repo-9.tar");

    let db = repo
        .store
        .ref_db()
        .expect("should be able to open packed-refs");

    assert_debug_snapshot!(db.lookup("refs/tags/v6.5.17"));
    assert_debug_snapshot!(db.lookup("refs/heads/main"));
}

#[test]
fn store_reference_loose() {
    let (repo, _dir) = include_repo!("fixtures/repos/repo-3.tar");

    assert_debug_snapshot!(repo.store.reference(&"refs/heads/main".into()));
    assert_debug_snapshot!(repo.store.reference(&"refs/heads/feature".into()));
}

#[test]
fn store_reference_packed() {
    let (repo, _dir) = include_repo!("fixtures/repos/repo-9.tar");

    assert_debug_snapshot!(repo.store.reference(&"refs/tags/v6.5.17".into()));
}

#[test]
fn store_reference_shadowing() {
    let (repo, _dir) = include_repo!("fixtures/repos/repo-5.tar");

    assert_debug_snapshot!(repo.store.reference(&"refs/heads/main".into()));
}

#[test]
fn store_reference_symbolic() {
    let (repo, _dir) = include_repo!("fixtures/repos/repo-9.tar");

    assert_debug_snapshot!(repo.store.reference(&"HEAD".into()));
    assert_debug_snapshot!(repo.store.reference(&"refs/heads/sym_1_7yjy".into()));
    assert_debug_snapshot!(repo.store.reference(&"refs/heads/sym_2_iqi5".into()));
}

#[test]
fn store_reference_missing() {
    let (repo_packed, _dir) = include_repo!("fixtures/repos/repo-9.tar");
    assert_debug_snapshot!(repo_packed.store.reference(&"refs/heads/ghost".into()));

    let (repo_loose_only, _dir) = include_repo!("fixtures/repos/repo-3.tar");
    assert_debug_snapshot!(repo_loose_only.store.reference(&"refs/heads/ghost".into()));
}
