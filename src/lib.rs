pub mod error;
pub use error::MinigitError;

pub mod object;
pub mod storage;

mod repo;
pub use repo::Repo;
