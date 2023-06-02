// #![warn(missing_docs)]
#![allow(unused_imports, unused_variables)]
//! title is
mod engines;
mod error;

pub use engines::{KvStore, KvsEngine, SledKvsEngine};
pub use error::{KvsError, Result};
