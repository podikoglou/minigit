mod common;

use insta::assert_debug_snapshot;

#[test]
fn store_packs_listing() {
    let (repo, _dir) = include_repo!("fixtures/repos/repo-5.tar");

    for pack in repo.store.packs() {
        assert_debug_snapshot!((&pack.name, &pack.objects_count));
    }
}
