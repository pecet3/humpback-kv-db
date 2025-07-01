mod cli;
mod core;
mod io_service;
mod object_service;
mod parser;
mod store;
use crate::{
    cli::{Cli, Commands},
    object_service::Kind,
    store::ObjectDescriptor,
};

use clap::Parser;
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

const DIR_PATH: &str = "./humpback-data";

fn main() {
    let mut core = core::Core::new().expect("Init error");

    let cli = Cli::parse();

    // Obsługa komend CLI
    match cli.command {
        Commands::GetObject { key } => match core.get(&key.to_string()) {
            Some(obj) => {
                println!("{:?}", obj)
            }
            _ => {
                println!("other")
            }
        },
        Commands::AddObject { key, data, kind } => {
            let parsed_data: Vec<u8> = data
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();
            core.add(&key.to_string(), parsed_data);
        }
    }
}
// Commands::GetAllGroups {} => {
//     let all_groups = groups.get_all_groups();
//     for group in all_groups {
//         println!("{:12} | {:<4}", "Name", "Sum");

//         println!("{:<12} | {:<4}", group.name, group.table_map.len());
//     }
// }
// Commands::AddGroup { name } => {
//     groups.add_group(&name).unwrap();
//     println!("Grupa '{}' została dodana.", name);
// }

//     let mut group_ref = groups.get_group_mut(&group).expect("Grupa nie istnieje");

//     group_ref.insert_object_data(&obj).expect("a");
//     println!("Obiekt dodany do grupy '{}'.", group);
// }

// Commands::GetGroup { group } => {
//     let group_ref = groups.get_group(&group).expect("Grupa nie istnieje");
//     println!("Grupa '{}':", group_ref.name);
//     println!("  Liczba obiektów: {}", group_ref.table_map.len());
// }
