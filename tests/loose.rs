use minigit::{
    object::{Object, hash::ObjectHash, tree::TreeEntry},
    storage::object::{LazyObject, loose::read_object},
};

#[test]
fn test_read_loose_blob() {
    let bytes = include_bytes!("fixtures/objects/blob-1");
    let object = read_object(&bytes[..]).expect("should be able to read loose blob object");

    let Object::Blob(blob) = &object else {
        panic!("expected Object::Blob, got {object:?}");
    };

    assert_eq!(blob.0.len(), 151);
    assert_eq!(
        object.hash().unwrap().to_string(),
        "be27a74ddcc0445b1710e25dd8df96fad679a10d"
    );
}

#[test]
fn test_read_loose_tree() {
    let bytes = include_bytes!("fixtures/objects/tree-1");
    let object = read_object(&bytes[..]).expect("should be able to read loose tree object");

    let Object::Tree(tree) = &object else {
        panic!("expected Object::Tree, got {object:?}");
    };

    assert_eq!(tree.entries.len(), 1);
    // assert_eq!(
    //     tree.entries.first_key_value(),
    //     Some((
    //         &String::from("README.md"),
    //         TreeEntry::new(0o100644 /* TODO */,)
    //     ))
    // );
}

#[test]
fn test_read_loose_commit() {
    let bytes = include_bytes!("fixtures/objects/commit-1");
    let object = read_object(&bytes[..]).expect("should be able to read loose commit object");

    let Object::Commit(commit) = &object else {
        panic!("expected Object::Commit, got {object:?}");
    };

    assert_eq!(
        commit.tree.to_string(),
        "7bfeab1d89aa800dff6acaae16b3433e7df44fa6"
    );
    assert_eq!(commit.author.0.name, "alex");
    assert_eq!(commit.author.0.email, "alex.podikoglou@gmail.com");
    assert_eq!(commit.committer.0.name, "alex");
    assert_eq!(commit.committer.0.email, "alex.podikoglou@gmail.com");
    assert_eq!(commit.description, "add readme\n");
}

#[test]
fn test_read_loose_non_root_commit() {
    let bytes = include_bytes!("fixtures/objects/commit-2");
    let object = read_object(&bytes[..]).expect("should be able to read loose commit object");

    let Object::Commit(commit) = &object else {
        panic!("expected Object::Commit, got {object:?}");
    };

    assert_eq!(
        commit.tree.to_string(),
        "f1596e78773e04b539660b10d75c02928ff38703"
    );
    assert_eq!(
        commit.parent,
        Some("497b458b242ad074b046386ec56b9f19361b2691".parse().unwrap())
    );
    assert_eq!(commit.author.0.name, "alex");
    assert_eq!(commit.author.0.email, "alex.podikoglou@gmail.com");
    assert_eq!(commit.committer.0.name, "alex");
    assert_eq!(commit.committer.0.email, "alex.podikoglou@gmail.com");
    assert_eq!(commit.description, "7133\n");
}
