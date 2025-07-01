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
    Add {
        key: String,
        data: String,
        kind: String,
    },

    Get {
        key: String,
    },
}
