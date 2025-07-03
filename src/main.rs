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
use std::{error::Error, str::FromStr};
use tokio::net::TcpListener;

const DIR_PATH: &str = "./humpback-data";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut core = core::Core::new().expect("Init error");

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Serwer TCP uruchomiony na 127.0.0.1:8080");

    loop {
        // Akceptuj nowe połączenia
        let (socket, addr) = listener.accept().await?;
        println!("Nowe połączenie od: {}", addr);

        // Obsłuż każde połączenie w osobnym zadaniu
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket).await {
                eprintln!("Błąd obsługi klienta: {}", e);
            }
        });
    }
}
