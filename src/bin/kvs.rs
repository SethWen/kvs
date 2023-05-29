use std::process::exit;

use clap::Parser;

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

fn main() {
    let opts = Opts::parse();
    match opts {
        Opts::Get(_args) => {
            eprintln!("unimplemented");
            exit(-1);
        }
        Opts::Set(_args) => {
            eprintln!("unimplemented");
            exit(-1);
        }
        Opts::Remove(_args) => {
            eprintln!("unimplemented");
            exit(-1);
        }
    }
}
