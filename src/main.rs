mod cli;
mod core;
mod io_service;
mod object_service;
mod parser;
use crate::{
    cli::{Cli, Commands},
    object_service::Kind,
};
use clap::Parser;
use std::str::FromStr;

const DIR_PATH: &str = "./humpback-data";

fn main() {
    let mut core = core::Core::new().expect("Init error");

    let cli = Cli::parse();

    // Obsługa komend CLI
    match cli.command {
        Commands::Get { key } => match core.get(&key.to_string()) {
            Some(obj) => {
                println!("{:?}", obj)
            }
            _ => {
                println!("other")
            }
        },
        Commands::Add { key, data, kind } => {
            let parsed_data: Vec<u8> = data.as_bytes().to_vec();
            println!("{:?}", parsed_data);
            let kind_enum = Kind::from_str(&kind).unwrap();
            core.set(&key.to_string(), kind_enum, parsed_data);
        }
    }
}
