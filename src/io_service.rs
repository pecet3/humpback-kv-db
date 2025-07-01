use crate::DIR_PATH;
use crate::object_service::Object;
use crate::store::ObjectDescriptor;

use std::cell::RefMut;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

pub fn get_raw_data_from_file(
    mut file: RefMut<File>,
    offset: u64,
    size: u64,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    file.seek(SeekFrom::Start(offset))?;

    let mut buffer = vec![0u8; size as usize];
    file.read_exact(&mut buffer)?;

    Ok(buffer)
}
pub fn save_object_in_file(
    obj: &Object,
    data: Vec<u8>,
    mut file: RefMut<File>,
) -> Result<u128, Box<dyn std::error::Error>> {
    let offset: u128 = file.seek(SeekFrom::End(0))? as u128;

    let header = create_header(obj.size);

    file.write_all(&header).expect("write header error");
    file.write_all(&data).expect("write data error");

    let padding = vec![0u8; 255];
    file.write_all(&padding).expect("write padding error");
    Ok(offset)
}
pub fn read_object_from_file(
    mut file: RefMut<File>,
    offset: usize,
    size: usize,
) -> Result<Vec<u8>, Box<dyn Error>> {
    file.seek(SeekFrom::Start(offset as u64))?;

    let header_size = 8;
    file.seek(SeekFrom::Current(header_size))?;

    let mut buffer = vec![0u8; size];
    file.read_exact(&mut buffer)?;

    Ok(buffer)
}

pub fn get_table_filename(prefix: &str) -> String {
    format!("{}/{}.Table.bindb", DIR_PATH, prefix)
}
pub fn get_data_filename(prefix: &str) -> String {
    format!("{}/{}.Data.bindb", DIR_PATH, prefix)
}
pub fn get_groups_list() -> String {
    format!("{}/groups.list", DIR_PATH)
}

fn create_header(length: usize) -> [u8; 8] {
    let mut header = [0u8; 8];

    header[0..4].copy_from_slice(&0xDEADBEEF_u32.to_be_bytes());

    let length_32 = length as u32;
    header[4..8].copy_from_slice(&length_32.to_be_bytes());

    header
}
