use crate::DIR_PATH;

pub fn get_table_filename(prefix: &str) -> String {
    format!("{}/{}.Table.bindb", DIR_PATH, prefix)
}
pub fn get_data_filename(prefix: &str) -> String {
    format!("{}/{}.Data.bindb", DIR_PATH, prefix)
}
pub fn get_groups_list() -> String {
    format!("{}/groups.list", DIR_PATH)
}
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

pub fn get_raw_data_from_file(
    key: &str,
    offset: u64,
    size: u64,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let filename = get_data_filename(key);
    let mut file = File::open(&filename)?;
    file.seek(SeekFrom::Start(offset))?;

    let mut buffer = vec![0u8; size as usize];
    file.read_exact(&mut buffer)?;

    Ok(buffer)
}
