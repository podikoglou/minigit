use minigit::{object::Object, storage::object::loose::read_object};

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
