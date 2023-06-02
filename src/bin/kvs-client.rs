use std::{env, error::Error};

use clap::{command, Parser};

#[derive(Parser)]
#[command(
    name = "kvs-client",
    bin_name = "kvs-client",
    author = env!("CARGO_PKG_AUTHORS"),
    version = env!("CARGO_PKG_VERSION"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
pub enum Opts {
    Get(GetArgs),
    Set(SetArgs),
    #[command(name = "rm")]
    Remove(RmArgs),
}

#[derive(clap::Args)]
#[command(about = "Get the string value of a given string key")]
pub struct GetArgs {
    #[arg(help = "A string key")]
    key: String,
}

#[derive(clap::Args)]
#[command(about = "Set the value of a string key to a string")]
pub struct SetArgs {
    #[arg(help = "A string key")]
    key: String,
    #[arg(help = "The string value of the key")]
    value: String,
}

#[derive(clap::Args)]
#[command(about = "Remove a given key")]
pub struct RmArgs {
    #[arg(help = "A string key")]
    key: String,
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::parse();
    let _store_dir = env::current_dir().unwrap();
    match opts {
        Opts::Get(_args) => {}
        Opts::Set(_args) => {}
        Opts::Remove(_args) => {}
    }
    Ok(())
}
