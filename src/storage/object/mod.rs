mod lazy;

pub mod loose;
pub mod packed;

pub use lazy::LazyObject;
pub use loose::bucket::ObjectsBucket;
