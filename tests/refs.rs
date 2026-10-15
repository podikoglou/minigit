use minigit::{MinigitError, storage::reference::LazyRef};

mod common;

#[test]
fn lazy_refs_iter_repo_3() {
    let (repo, _dir) = include_repo!("fixtures/repos/repo-3.tar");
    let refs: Vec<Result<LazyRef, MinigitError>> = repo.store.lazy_refs().collect();
    let refs: Vec<_> = refs
        .into_iter()
        .map(|x| x.map(|x| (x.name, x.pointer)))
        .collect();

    insta::assert_debug_snapshot!(refs);
}

#[test]
fn lazy_refs_iter_repo_4() {
    let (repo, _dir) = include_repo!("fixtures/repos/repo-4.tar");
    let refs: Vec<Result<LazyRef, MinigitError>> = repo.store.lazy_refs().collect();
    let refs: Vec<_> = refs
        .into_iter()
        .map(|x| x.map(|x| (x.name, x.pointer)))
        .collect();

    insta::assert_debug_snapshot!(refs);
}
