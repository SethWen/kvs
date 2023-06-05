// #![warn(missing_docs)]
#![allow(unused_imports, unused_variables)]
//! title is
mod client;
mod common;
mod engines;
mod error;
mod server;

pub use client::Client;
pub use common::*;
pub use engines::{KvStore, KvsEngine, SledKvsEngine};
pub use error::{KvsError, Result};
pub use server::Server;
