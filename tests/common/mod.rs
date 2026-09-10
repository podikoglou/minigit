#[macro_export]
macro_rules! include_repo {
    ($path:literal) => {{
        let dir = tempfile::tempdir().unwrap();
        let bytes = include_bytes!($path);
        let vec: Vec<u8> = bytes.into();
        let cursor = std::io::Cursor::new(vec);

        let mut archive = tar::Archive::new(cursor);
        archive.unpack(&dir).unwrap();

        dir
    }};
}
