#[allow(dead_code)]
pub fn unpack_repo(bytes: &[u8]) -> (minigit::Repo, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();

    let cursor = std::io::Cursor::new(bytes);
    let mut archive = tar::Archive::new(cursor);
    archive.unpack(&dir).unwrap();

    (
        minigit::Repo::open(dir.path()).expect("should be able to open repo"),
        dir,
    )
}

#[macro_export]
macro_rules! include_repo {
    ($path:literal) => {{
        $crate::common::unpack_repo(include_bytes!($path))
    }};
}
