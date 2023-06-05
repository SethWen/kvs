use std::{env::current_dir, error::Error, process};

use clap::Parser;
use kvs::{KvStore, KvsEngine};

#[derive(Parser)]
#[command(
    name = env!("CARGO_PKG_NAME"),
    bin_name = env!("CARGO_PKG_NAME"),
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

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::parse();
    let store_dir = current_dir().unwrap();
    let mut store = KvStore::open(store_dir).unwrap();
    match opts {
        Opts::Get(args) => {
            match store.get(args.key)? {
                Some(value) => println!("{value}"),
                None => {
                    println!("Key not found");
                    process::exit(0);
                }
            };
        }
        Opts::Set(args) => {
            store.set(args.key, args.value)?;
        }
        Opts::Remove(args) => match store.remove(args.key) {
            Ok(_) => {}
            Err(kvs::KvsError::KeyNotFound) => {
                println!("Key not found");
                process::exit(-1);
            }
            _ => todo!(),
        },
    }
    Ok(())
}
