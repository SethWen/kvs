// #![warn(missing_docs)]
#![allow(unused_imports, unused_variables)]
//! title is
mod client;
mod engines;
mod error;
mod server;
mod common;

pub use client::Client;
pub use engines::{KvStore, KvsEngine, SledKvsEngine};
pub use error::{KvsError, Result};
pub use server::Server;
pub use common::*;
