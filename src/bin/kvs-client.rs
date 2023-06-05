use std::{env, error::Error, net::SocketAddr};

use clap::{command, Parser};
use kvs::Client;

#[derive(Parser)]
#[command(
    name = "kvs-client",
    bin_name = "kvs-client",
    author = env!("CARGO_PKG_AUTHORS"),
    version = env!("CARGO_PKG_VERSION"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
pub enum Opts {
    // #[arg(
    //     short,
    //     long,
    //     help = "Sets the listening address",
    //     value_name = "IP:PORT",
    //     default_value = "127.0.0.1:4000"
    // )]
    // addr: SocketAddr,
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
    #[arg(
        short,
        long,
        help = "Sets the listening address",
        value_name = "IP:PORT",
        default_value = "127.0.0.1:4000"
    )]
    addr: SocketAddr,
}

#[derive(clap::Args)]
#[command(about = "Set the value of a string key to a string")]
pub struct SetArgs {
    #[arg(help = "A string key")]
    key: String,
    #[arg(help = "The string value of the key")]
    value: String,
    #[arg(
        short,
        long,
        help = "Sets the listening address",
        value_name = "IP:PORT",
        default_value = "127.0.0.1:4000"
    )]
    addr: SocketAddr,
}

#[derive(clap::Args)]
#[command(about = "Remove a given key")]
pub struct RmArgs {
    #[arg(help = "A string key")]
    key: String,
    #[arg(
        short,
        long,
        help = "Sets the listening address",
        value_name = "IP:PORT",
        default_value = "127.0.0.1:4000"
    )]
    addr: SocketAddr,
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::parse();
    let _store_dir = env::current_dir().unwrap();
    match opts {
        Opts::Get(args) => {
            let mut client = Client::connect(args.addr)?;
            if let Some(value) = client.get(args.key)? {
                println!("{}", value);
            } else {
                println!("Key not found");
            }
        }
        Opts::Set(args) => {
            // println!("set: {}:{}", args.key, args.value);
            let mut client = Client::connect(args.addr)?;
            client.set(args.key, args.value)?
        }
        Opts::Remove(args) => {
            // println!("remove: {}", args.key);
            let mut client = Client::connect(args.addr)?;
            client.remove(args.key)?;
        }
    }
    Ok(())
}
