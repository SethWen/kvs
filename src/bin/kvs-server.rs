use std::{env, error::Error, fs, net::SocketAddr};

use clap::{
    builder::{IntoResettable, OsStr, Resettable},
    command, Parser, ValueEnum,
};

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, ValueEnum)]
enum Engine {
    kvs,
    sled,
}

impl IntoResettable<OsStr> for Engine {
    fn into_resettable(self) -> Resettable<OsStr> {
        match self {
            Engine::kvs => Resettable::Value("kvs".into()),
            Engine::sled => Resettable::Value("slet".into()),
        }
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "kvs-server",
    bin_name = "kvs-server",
    author = env!("CARGO_PKG_AUTHORS"),
    version = env!("CARGO_PKG_VERSION"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
pub struct Opts {
    #[arg(
        short,
        long,
        help = "Sets the listening address",
        value_name = "IP:PORT",
        default_value = "127.0.0.1:4000"
    )]
    addr: SocketAddr,

    #[arg(
        short,
        long,
        help = "Sets the storage engine",
        value_name = "ENGINE-NAME",
        default_value = Engine::kvs,
    )]
    engine: Engine,
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::parse();
    println!("opts: {:?}", opts);
    eprintln!("kvs-server {}", env!("CARGO_PKG_VERSION"));
    eprintln!("Storage engine: {:?}", opts.engine);
    eprintln!("Listening on {}", opts.addr);

    // write engine to engine file
    fs::write(
        env::current_dir()?.join("engine"),
        format!("{:?}", opts.engine),
    )?;
    // let _store_dir = env::current_dir().unwrap();
    Ok(())
}
