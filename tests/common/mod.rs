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

#[allow(dead_code)]
pub fn unpack_git2_repo(bytes: &[u8]) -> (git2::Repository, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();

    let cursor = std::io::Cursor::new(bytes);
    let mut archive = tar::Archive::new(cursor);
    archive.unpack(&dir).unwrap();

    (
        git2::Repository::open(dir.path()).expect("should be able to open repo"),
        dir,
    )
}

#[macro_export]
macro_rules! include_repo {
    ($path:literal) => {{ $crate::common::unpack_repo(include_bytes!($path)) }};
}

#[macro_export]
macro_rules! include_repo_git2 {
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

        (
            git2::Repository::open(dir.path()).expect("should be able to open repo"),
            dir,
        )
    }};
}
