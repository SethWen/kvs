// #![warn(missing_docs)]

//! title is
mod error;
mod kv;

pub use error::{Result, KvsError};
pub use kv::KvStore;

pub fn do_sth() {}
