use crate::DIR_PATH;

use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::{Arc, Mutex};

const HEADER_SIZE: i64 = 8;

pub fn update_chunk_in_file(
    offset: u64,
    data: Vec<u8>,
    file: Arc<Mutex<File>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut file = file.lock().unwrap();
    file.seek(SeekFrom::Start(offset))?;
    let meta = file.metadata()?;
    println!("meta len {}", meta.len());
    file.write_all(&data)?;
    file.flush()?;
    Ok(())
}

pub fn save_desc_in_file(
    mut data: Vec<u8>,
    file: Arc<Mutex<File>>,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let mut file = file.lock().unwrap();
    let offset: u64 = file.seek(SeekFrom::End(0))? as u64;

    let offset_bytes = offset.to_le_bytes();

    let start = data.len() - 8;
    data[start..].copy_from_slice(&offset_bytes);

    file.write_all(&data).expect("write data error");

    Ok(offset)
}

pub fn save_object_in_file(
    data: &Vec<u8>,
    file: Arc<Mutex<File>>,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let mut file = file.lock().unwrap();

    let offset: u64 = file.seek(SeekFrom::End(0))? as u64;

    let header = create_header(data.len() as u64);

    file.write_all(&header).expect("write header error");
    file.write_all(&data).expect("write data error");

    Ok(offset)
}
pub fn read_object_from_file(
    file: Arc<Mutex<File>>,
    offset: u64,
    size: u64,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut file = file.lock().unwrap();

    file.seek(SeekFrom::Start(offset as u64))?;

    file.seek(SeekFrom::Current(HEADER_SIZE as i64))?;

    let mut buffer = vec![0u8; size as usize];
    file.read_exact(&mut buffer)?;

    Ok(buffer)
}

pub fn get_desc_filename(prefix: &str) -> String {
    format!("{}/{}.Desc.bindb", DIR_PATH, prefix)
}
pub fn get_data_filename(prefix: &str) -> String {
    format!("{}/{}.Data.bindb", DIR_PATH, prefix)
}

fn create_header(length: u64) -> [u8; 8] {
    let mut header = [0u8; 8];

    header[0..4].copy_from_slice(&0xDEADBEEF_u32.to_be_bytes());

    let length_32 = length as u32;
    header[4..8].copy_from_slice(&length_32.to_be_bytes());

    header
}
