mod core;
mod io_service;
mod object_service;
use crate::{core::Core, object_service::Kind};
use std::{error::Error, str::FromStr, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
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
                let start = std::time::Instant::now();
                let data = core.get(key).await;
                let duration = start.elapsed();
                println!("GET completed in {:.2?}", duration);
                match data {
                    Some(data) => {
                        writer.write_all(&data).await?;
                        writer.write_all(b"\n").await?;
                    }
                    None => {
                        writer.write_all(b"> NOT_FOUND\n").await?;
                    }
                }
            }
            ["DELETE", key] => {
                let start = std::time::Instant::now();
                match core.delete_soft(key).await {
                    Ok(_) => {
                        writer.write_all(b"> SUCCESS\n").await?;
                    }
                    Err(_) => {
                        writer.write_all(b"> NOT_FOUND\n").await?;
                    }
                }
                let duration = start.elapsed();
                println!("DELETE completed in {:.2?}", duration);
            }
            ["SET", key, kind] => {
                let kind = Kind::from_str(kind).unwrap();
                writer.write_all(b"> WRITE DATA\n").await?;
                let mut data_buf = vec![0; 1024 * 4];
                let data_size = buf_reader.read(&mut data_buf).await?;
                data_buf.truncate(data_size);
                let start = std::time::Instant::now();
                core.set(key, kind, data_buf).await;
                let duration = start.elapsed();
                println!("SET completed in {:.2?} ({} bytes)", duration, data_size);
                writer.write_all(b"> SUCCESS\n").await?;
            }
            ["LIST"] => {
                let start = std::time::Instant::now();

                match core.list().await {
                    Ok(list) => {
                        for chunk in list.chunks(2) {
                            let line = chunk
                                .iter()
                                .map(|element| {
                                    format!(
                                        "[{}] <{}> size: {}",
                                        element.key,
                                        element.kind.to_string().to_uppercase(),
                                        element.size
                                    )
                                })
                                .collect::<Vec<_>>()
                                .join(" | ");
                            writer.write_all(format!("{}\n", line).as_bytes()).await?;
                        }
                    }
                    Err(_) => {
                        writer.write_all(b"> ERR Unable to list objects\n").await?;
                    }
                }
                let duration = start.elapsed();
                println!("LIST completed in {:.2?}", duration);
            }
            ["LIST_TYPE", kind] => {
                let start = std::time::Instant::now();
                let kind_enum = match Kind::from_str(&kind) {
                    Ok(k) => k,
                    Err(_) => {
                        writer.write_all(b"> ERR Invalid type\n").await?;
                        continue;
                    }
                };

                match core.list_by_kind(kind_enum).await {
                    Ok(list) => {
                        for chunk in list.chunks(2) {
                            let line = chunk
                                .iter()
                                .map(|element| format!("[{}] size: {}", element.key, element.size))
                                .collect::<Vec<_>>()
                                .join(" | ");
                            writer.write_all(format!("{}\n", line).as_bytes()).await?;
                        }
                    }
                    Err(_) => {
                        writer.write_all(b"> ERR Unable to list objects\n").await?;
                    }
                }
                let duration = start.elapsed();
                println!("LIST completed in {:.2?}", duration);
            }
            _ => {
                writer
                    .write_all(
                        b"> ERR Invalid command. Use one of: \
                    GET <key> | SET <key> <type> | LIST | LIST_TYPE <type>\n",
                    )
                    .await?;
            }
        }
    }
    Ok(())
}
