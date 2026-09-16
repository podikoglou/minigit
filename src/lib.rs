pub mod error;
pub use error::MinigitError;

pub mod object;
pub mod storage;

pub mod fs;
pub mod identity;
pub mod time;

mod repo;
pub use repo::Repo;
