use std::{
    cell::RefCell, collections::HashMap, fs::{self, File, OpenOptions}, io::{Read, Seek, SeekFrom, Write}, path::Path, sync::{Arc, Mutex}, time::{}
};

#[derive(Debug, Clone)]
pub enum Kind {
    Number,
    Boolean,
    String,
    Json,
    Struct,
}

use std::str::FromStr;

use crate::io_service;

impl FromStr for Kind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "number" => Ok(Kind::Number),
            "boolean" => Ok(Kind::Boolean),
            "string" => Ok(Kind::String),
            "json" => Ok(Kind::Json),
            "struct" => Ok(Kind::Struct),
            _ => Err(()),
        }
    }
}
impl Kind {
    pub fn from_u8(value: u8) -> Kind {
        match value {
            0 => Kind::Number,
            1 => Kind::Boolean,
            2 => Kind::String,
            3 => Kind::Json,
            4 => Kind::Struct,
            _ => Kind::Struct, 
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Kind::Number => 0,
            Kind::Boolean => 1,
            Kind::String => 2,
            Kind::Json => 3,
            Kind::Struct => 4,
        }
    }
}


const DIR_PATH: &str = "./humpback-data";


#[derive(Debug, Clone)]
pub struct Object_descriptor {
    pub data: Vec<u8>,
    pub created_at: u128,
    pub updated_at: u128,
    pub last_opened_at: u128,
    pub next_offset: usize,
    pub kind: Kind,
    pub offset: usize,
    pub size: usize,
    pub key: String,
    pub is_mem_storage: bool,
    pub header: [u8; 16],
    pub columns: Vec<usize>,
   pub free_bytes: usize,
}

impl Object_descriptor {
    pub fn get_header(&self) -> [u8; 16] {
        let mut header = [0u8; 16];

        let mut key_part = [0u8; 4];
        let key_bytes = self.key.as_bytes();
        let len = key_bytes.len().min(4);
        key_part[0..len].copy_from_slice(&key_bytes[0..len]);
        header[0..4].copy_from_slice(&key_part);

        header[4..8].copy_from_slice(&(self.created_at as u32).to_le_bytes());

        header[8..12].copy_from_slice(&(self.data.len() as u32).to_le_bytes());

        header[12..16].copy_from_slice(&(self.next_offset as u32).to_le_bytes());

        header
    }

    pub fn to_fixed_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // 1. key: 128 bajtów
        let key_bytes = self.key.as_bytes();
        let mut fixed_key = [0u8; 128];
        let len = key_bytes.len().min(128);
        fixed_key[..len].copy_from_slice(&key_bytes[..len]);
        bytes.extend_from_slice(&fixed_key);

        // 2. created_at, updated_at, last_opened_at: u128 x 3 (16 x 3 = 48)
        bytes.extend_from_slice(&self.created_at.to_le_bytes());
        bytes.extend_from_slice(&self.updated_at.to_le_bytes());
        bytes.extend_from_slice(&self.last_opened_at.to_le_bytes());

        // 3. next_offset, offset, size, free_bytes: usize x 4
        bytes.extend_from_slice(&(self.next_offset as u64).to_le_bytes());
        bytes.extend_from_slice(&(self.offset as u64).to_le_bytes());
        bytes.extend_from_slice(&(self.size as u64).to_le_bytes());
        bytes.extend_from_slice(&(self.free_bytes as u64).to_le_bytes());

        // 4. kind (u8)
        bytes.push(self.kind.clone() as u8);

        // 5. is_mem_storage (bool as u8)
        bytes.push(self.is_mem_storage as u8);

        // 6. header: [u8; 16]
        bytes.extend_from_slice(&self.header);

        // 7. columns: serialize length + content
        bytes.extend_from_slice(&(self.columns.len() as u32).to_le_bytes());
        for column in &self.columns {
            bytes.extend_from_slice(&(*column as u64).to_le_bytes());
        }

        // NOTE: self.data is skipped

        bytes
    }
    pub fn from_fixed_bytes(buf: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut index = 0;

        // 1. key (128 bytes)
        let key_bytes = &buf[index..index + 128];
        let key = String::from_utf8(key_bytes.to_vec())?.trim_end_matches(char::from(0)).to_string();
        index += 128;

        // 2. timestamps (3 * u128)
        let created_at = u128::from_le_bytes(buf[index..index + 16].try_into()?);
        index += 16;
        let updated_at = u128::from_le_bytes(buf[index..index + 16].try_into()?);
        index += 16;
        let last_opened_at = u128::from_le_bytes(buf[index..index + 16].try_into()?);
        index += 16;

        // 3. offsets (4 * u64)
        let next_offset = u64::from_le_bytes(buf[index..index + 8].try_into()?);
        index += 8;
        let offset = u64::from_le_bytes(buf[index..index + 8].try_into()?);
        index += 8;
        let size = u64::from_le_bytes(buf[index..index + 8].try_into()?);
        index += 8;
        let free_bytes = u64::from_le_bytes(buf[index..index + 8].try_into()?);
        index += 8;

        // 4. kind (u8)
        let kind = Kind::from_u8(buf[index]);
        index += 1;

        // 5. is_mem_storage (u8)
        let is_mem_storage = buf[index] != 0;
        index += 1;

        // 6. header (16 bytes)
        let header: [u8; 16] = buf[index..index + 16].try_into()?;
        index += 16;

        // 7. columns length (u32)
        let columns_len = u32::from_le_bytes(buf[index..index + 4].try_into()?);
        index += 4;

        // 8. columns content (u64 * N)
        let mut columns = Vec::with_capacity(columns_len as usize);
        for _ in 0..columns_len {
            let col = u64::from_le_bytes(buf[index..index + 8].try_into()?);
            index += 8;
            columns.push(col as usize);
        }

        Ok(Object_descriptor {
            key,
            created_at,
            updated_at,
            last_opened_at,
            next_offset: next_offset as usize,
            offset: offset as usize,
            size: size as usize,
            free_bytes: free_bytes as usize,
            kind,
            is_mem_storage,
            header,
            columns,
            data: vec![], // data intentionally left empty
        })
    }
}


#[derive(Debug)]
pub struct Group {
    pub name: String,
    pub data_map: HashMap<String, Vec<u8>>,
    pub table_map: HashMap<String, Object_descriptor>,
    pub data_file: RefCell<File>,
    pub table_file: RefCell<File>,
}

impl Group {
    pub fn new(name: &String) -> Result<Group, Box<dyn std::error::Error>> {
        let objs = io_service::IoService::get_data_filename(name);
        let objs_tab = io_service::IoService::get_table_filename(name);

        fs::create_dir_all(DIR_PATH)?;

        if !Path::new(&objs).exists() {
            File::create(&objs)?;
        }
        if !Path::new(&objs_tab).exists() {
            File::create(&objs_tab)?;
        }

        let data_file = OpenOptions::new()
            .read(true)
            .write(true)
            .append(true) 
            .create(true)
            .open(&objs)?;

        let table_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true) 
            .open(&objs_tab)?;

        Ok(Group {
            name: name.clone(), 
            data_map: HashMap::new(),
            table_map: HashMap::new(),
            data_file: RefCell::new(data_file),
            table_file: RefCell::new(table_file),
        })
    }

    pub fn insert_object_data(
        &mut self,
        obj: &Object_descriptor,
    ) -> Result<u128, Box<dyn std::error::Error>> {
        // Teraz możesz bezpośrednio używać data_file
        let mut file = self.data_file.borrow_mut();
        let offset: u128 = file.seek(SeekFrom::End(0))? as u128;

        let header = obj.get_header();

        file.write_all(&header)?;
        file.write_all(&obj.data)?;

        let padding = vec![0u8; 255];
        file.write_all(&padding)?;
        
        self.table_map.insert(obj.key.clone(), obj.clone());

        Ok(offset)
    }

  
   

    pub fn read_Object_descriptor(
        &self,
        offset: u64,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        let mut file = self.data_file.borrow_mut();

        file.seek(SeekFrom::Start(offset))?;

        // Read header (16 bytes)
        let mut header = [0u8; 16];
        if file.read_exact(&mut header).is_err() {
            return Ok(None); // Return None if unable to read header, indicating end of data or invalid offset
        }

        // Extract data length from header (assuming bytes 4-7 hold data length)
        let data_length = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;

        // Read data + padding
        let mut data_with_padding = vec![0u8; data_length + 255];
        file.read_exact(&mut data_with_padding)?;

        // Only return the actual data part, not the padding
        let actual_data = data_with_padding[..data_length].to_vec();

        Ok(Some(actual_data))
    }

    

    pub fn get_object_descriptor_by_key(
        &self,
        key: &str,
    ) -> Result<Option<&Object_descriptor>, Box<dyn std::error::Error>> {
        if let Some(Object_descriptor) = self.table_map.get(key) {
            return Ok(Some(Object_descriptor));
        }

        Ok(None)
    }

}

