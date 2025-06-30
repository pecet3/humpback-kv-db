use std::collections::HashMap;

use clap::{Parser, Subcommand, builder::Str};

#[derive(Parser)]
#[command(name = "Humpback DB")]
#[command(about = "Prosta CLI-baza danych z grupami i obiektami", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Dodaj nową grupę
    AddGroup {
        /// Nazwa grupy
        name: String,
    },

    /// Dodaj obiekt do grupy
    AddObject {
        /// Nazwa grupy
        group: String,
        /// Klucz obiektu
        key: String,
        /// Dane obiektu (np. "1,2,3")
        data: String,
        kind: String,
    },

    GetObject {
        key: String,
        group: Option<String>,
    },
    // /// Wypisz obiekty w grupie
    // ListObjects {
    //     /// Nazwa grupy
    //     objects: Vec<String>,
    // },
    GetGroup {
        group: String,
    },

    GetAllGroups {},
}
