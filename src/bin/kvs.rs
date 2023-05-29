use std::process::exit;

use clap::{Arg, Command};

fn main() {
    let _store = kvs::KvStore::new();
    let cmd = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .subcommand(
            Command::new("set")
                .about("Set the value of a string key to a string")
                .arg(Arg::new("key").help("A string key").required(true))
                .arg(
                    Arg::new("value")
                        .help("The string value of the key")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("get")
                .about("Get the string value of a given string key")
                .arg(Arg::new("key").help("A string key").required(true)),
        )
        .subcommand(
            Command::new("rm")
                .about("Remove a given key")
                .arg(Arg::new("key").help("A string key").required(true)),
        );

    let matches = cmd.get_matches();
    match matches.subcommand() {
        Some(("set", _args)) => {
            // let key = args.get_one::<String>("key").unwrap();
            // let value = args.get_one::<String>("value").unwrap();
            // let mut store = kvs::KvStore::new();
            // store.set(key.to_owned(), value.to_owned());
            eprintln!("unimplemented");
            exit(-1);
        }
        Some(("get", _args)) => {
            // let key = args.get_one::<String>("key").unwrap();
            // let store = kvs::KvStore::new();
            // match store.get(key.to_owned()) {
            //     Some(value) => println!("{}", value),
            //     None => println!("nil"),
            // }
            eprintln!("unimplemented");
            exit(-1);
        }
        Some(("rm", _args)) => {
            // let key = args.get_one::<String>("key").unwrap();
            // let mut store = kvs::KvStore::new();
            // store.remove(key.to_owned());
            eprintln!("unimplemented");
            exit(-1);
        }
        _ => unimplemented!(),
    }
}
