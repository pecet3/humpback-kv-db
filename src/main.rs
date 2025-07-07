mod cli;
mod core;
mod io_service;
mod object_service;
mod parser;
use crate::{
    cli::{Cli, Commands},
    core::Core,
    object_service::Kind,
};
use clap::Parser;
use std::{error::Error, str::FromStr, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

const DIR_PATH: &str = "./humpback-data";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let core = core::Core::new().expect("Init error");
    let arc_core = Arc::new(core);
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Humpback KV Database is listening on 127.0.0.1:8080");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from: {}", addr);
        let core = Arc::clone(&arc_core);
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, core).await {
                eprintln!("Connection error: {}", e);
            }
        });
    }
}

async fn handle_client(socket: TcpStream, core: Arc<Core>) -> Result<(), Box<dyn Error>> {
    let (reader, mut writer) = socket.into_split();
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();
    loop {
        line.clear();
        let byte_read = buf_reader.read_line(&mut line).await?;
        if byte_read == 0 {
            break;
        }
        let trimmed = line.trim();
        let parts: Vec<&str> = trimmed.splitn(3, ' ').collect();

        match parts.as_slice() {
            ["GET", key] => {
                let data = core.clone().get(key).await;
                match data {
                    Some(data) => {
                        writer.write_all(b"OK\n").await?;
                        writer.write_all(&data).await?;
                        writer.write_all(b"\n").await?;
                    }
                    None => {
                        writer.write_all(b"NOT_FOUND\n").await?;
                    }
                }
            }

            _ => {
                writer
                    .write_all(b"ERR Invalid command. Use GET <key> or SET <key> <kind>\n")
                    .await?;
            }
        }
    }
    Ok(())
}
