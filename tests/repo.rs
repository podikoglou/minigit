use std::fs::File;

use tinygit::Repo;

#[test]
fn repo_new_dir_structure() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path();

    let repo = Repo::new(path).expect("should be able to create repository");

    // assert .git structure
    assert!(path.join(".git").exists(), ".git should exist");
    assert!(
        path.join(".git/objects").exists(),
        ".git/objects should exist"
    );
    assert!(
        path.join(".git/objects/pack").exists(),
        ".git/objects/pack should exist"
    );
}

#[test]
fn repo_new_create_dir() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path();
    let path = path.join("some-subdir");

    let _ = Repo::new(path).expect("should be able to create directory for repository");
}

#[test]
fn repo_new_reinitialize() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path();

    // first repo
    let _ = Repo::new(path).expect("should be able to create repository");

    File::create(path.join("foo.bar")).expect("should be able to create file");

    // reinitialize repo
    let _ = Repo::new(path).expect("should be able to re-create repository");

    assert!(
        path.join("foo.bar").exists(),
        "reinitialization shouldn't remove repo files"
    );
}
