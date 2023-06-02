// #![warn(missing_docs)]
#![allow(unused_imports, unused_variables)]
//! title is
mod error;
mod engines;

pub use error::{Result, KvsError};
pub use engines::{KvsEngine, KvStore, SledKvsEngine};
