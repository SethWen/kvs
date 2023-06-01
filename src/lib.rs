// #![warn(missing_docs)]
#![allow(unused_imports, unused_variables)]
//! title is
mod error;
mod kv;
// mod kv_example;

pub use error::{Result, KvsError};
pub use kv::KvStore;

pub fn do_sth() {}
