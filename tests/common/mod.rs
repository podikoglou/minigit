#[macro_export]
macro_rules! include_repo {
    ($path:literal) => {{
        // create temp directory to unpack repo
        let dir = tempfile::tempdir().unwrap();

        // tar bytes of the repo
        let bytes = include_bytes!($path);
        let vec: Vec<u8> = bytes.into();
        let cursor = std::io::Cursor::new(vec);

        // open tar and unpack it into the dir
        let mut archive = tar::Archive::new(cursor);
        archive.unpack(&dir).unwrap();

        dir
    }};
}
