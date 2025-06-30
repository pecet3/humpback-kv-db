mod cli;
mod core;
mod groups;
mod io_service;
mod objects;
mod parser;

use crate::cli::{Cli, Commands};
use crate::groups::ObjectStore;
use crate::objects::{Kind, Object_descriptor};

use clap::Parser;
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

const DIR_PATH: &str = "./humpback-data";

fn main() {
    let mut core = core::Core::new();
    let mut groups = ObjectStore::new();

    let cli = Cli::parse();

    // Obsługa komend CLI
    match cli.command {
        Commands::GetObject { key, group } => {
            let name = group.clone().unwrap();
            if core.store.groups.contains_key(&name) {
                match core.get_object_by_key_and_group(group.unwrap().clone().as_str(), &key) {
                    Ok(Some(obj)) => {
                        println!(
                            "Znaleziono obiekt o kluczu '{}' w grupie '{:?}': {:?}",
                            key, name, obj
                        );
                    }
                    Ok(None) => {
                        println!(
                            "Obiekt o kluczu '{}' nie istnieje w grupie '{:?}'",
                            key, name
                        );
                    }
                    Err(e) => {
                        println!("{:?}", e)
                    }
                }
            }
        }
        Commands::GetAllGroups {} => {
            let all_groups = groups.get_all_groups();
            for group in all_groups {
                println!("{:12} | {:<4}", "Name", "Sum");

                println!("{:<12} | {:<4}", group.name, group.table_map.len());
            }
        }
        Commands::AddGroup { name } => {
            groups.add_group(&name).unwrap();
            println!("Grupa '{}' została dodana.", name);
        }

        Commands::AddObject {
            group,
            key,
            data,
            kind,
        } => {
            // Pobierz referencję do grupy

            // Parsuj dane wejściowe z ciągu np. "1,2,3"
            let parsed_data: Vec<u8> = data
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();

            // Aktualny timestamp
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();

            // Utwórz obiekt
            let mut obj = Object_descriptor {
                data: parsed_data,
                created_at: now,
                updated_at: now,
                last_opened_at: now,
                next_offset: 0,
                kind: Kind::Number,
                offset: 0,
                size: 0,
                key,
                is_mem_storage: true,
                header: [0; 16],
                columns: vec![],
                free_bytes: 0,
            };

            match kind.to_lowercase().as_str() {
                "number" => {
                    obj.kind = Kind::Number;
                }
                "bool" => {
                    obj.kind = Kind::Boolean;
                }
                "string" => {
                    obj.kind = Kind::String;
                }
                "json" => {
                    obj.kind = Kind::Json;
                }
                "struct" => {
                    obj.kind = Kind::Struct;
                }
                _ => {
                    println!("Nieznany typ: {}", kind);
                }
            }

            let mut group_ref = groups.get_group_mut(&group).expect("Grupa nie istnieje");

            group_ref.insert_object_data(&obj).expect("a");
            println!("Obiekt dodany do grupy '{}'.", group);
        }

        Commands::GetGroup { group } => {
            let group_ref = groups.get_group(&group).expect("Grupa nie istnieje");
            println!("Grupa '{}':", group_ref.name);
            println!("  Liczba obiektów: {}", group_ref.table_map.len());
        }
    }
}
