#![feature(once_cell_try)]
pub mod error;
pub use error::{MinigitError, ParseError, ParserContext};

pub mod object;
pub mod storage;

pub mod fs;
pub mod identity;
pub mod time;

pub mod parsing;

mod repo;
pub use repo::Repo;
