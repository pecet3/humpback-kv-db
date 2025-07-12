use std::{error::Error, io::Write, str::FromStr, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    signal,
    sync::Notify,
};

use crate::database::{core::Core, objects::Kind};

#[tokio::main]
pub async fn run(core: Arc<Core>) -> Result<(), Box<dyn Error>> {
    let notify_shutdown = Arc::new(Notify::new());

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Humpback KV Database is listening on 127.0.0.1:8080");

    let shutdown_notify = Arc::clone(&notify_shutdown);
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen ctrl+c");
        shutdown_notify.notify_waiters();
    });

    loop {
        tokio::select! {
            Ok((socket, addr)) = listener.accept() => {
                println!("New connection from: {}", addr);
                let core = Arc::clone(&core);
                tokio::spawn(async move {
                    if let Err(e) = handle_client(socket, core).await {
                        eprintln!("Connection error: {}", e);
                    }
                });
            }
            _ = notify_shutdown.notified() => {
                break;
            }
        }
    }
    println!("Exit signal received\nInitiating graceful shutdown...");
    tokio::time::sleep(Duration::from_millis(500)).await;
    let exit_core = Arc::clone(&core);

    let mut desc_file = exit_core.desc_file.lock().unwrap();
    desc_file.flush()?;

    let mut data_file = exit_core.data_file.lock().unwrap();
    data_file.flush()?;

    println!("All data flushed to disk");
    println!("Resources released");
    Ok(())
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
                let data = core.get_async(key).await;
                let duration = start.elapsed();
                println!("GET completed in {:.2?}", duration);
                match data {
                    Some(data) => {
                        writer.write_all(b"> SUCCESS\n").await?;
                        writer.write_all(&data).await?;
                        writer.write_all(b"\n").await?;
                    }
                    None => {
                        writer.write_all(b"> NOT FOUND\n").await?;
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
                        writer.write_all(b"> NOT FOUND\n").await?;
                    }
                }
                let duration = start.elapsed();
                println!("DELETE completed in {:.2?}", duration);
            }
            ["SET", key, kind] => {
                if key.len() > 256 {
                    writer
                        .write_all(b"> ERR Key is too long. Max key length - 256 bytes\n")
                        .await?;
                    continue;
                }
                let kind = Kind::from_str(kind).unwrap();
                writer.write_all(b"> WRITE DATA\n").await?;
                let kind = match Kind::from_str(&kind.to_string()) {
                    Ok(k) => k,
                    Err(_) => {
                        writer.write_all(b"> ERR Unknown kind\n").await?;
                        continue;
                    }
                };
                let buf_size = match kind {
                    Kind::Number => 16,
                    Kind::Boolean => 4,
                    Kind::String => 1024 * 16,    // 16 KB
                    Kind::Json => 1024 * 64,      // 64 KB
                    Kind::Blob => 1024 * 256 * 4, // 1 Mb
                };
                let mut data_buf = vec![0; buf_size];
                let data_size = buf_reader.read(&mut data_buf).await?;
                data_buf.truncate(data_size);
                let start = std::time::Instant::now();
                core.set_async(key, kind, data_buf).await;
                let duration = start.elapsed();
                println!("SET completed in {:.2?} ({} bytes)", duration, data_size);
                writer.write_all(b"> SUCCESS\n").await?;
            }
            ["LIST"] => {
                let start = std::time::Instant::now();

                match core.list().await {
                    Ok(list) => {
                        if list.len() <= 0 {
                            writer.write_all(b"> No objects\n").await?;
                            continue;
                        }
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
                        if list.len() <= 0 {
                            writer.write_all(b"> No objects\n").await?;
                            continue;
                        }
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
