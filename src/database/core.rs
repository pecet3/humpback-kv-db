use std::{
    error::Error,
    fs::{self, File, OpenOptions},
    path::Path,
    sync::{Arc, Mutex},
};

use crate::{
    DIR_PATH,
    database::{
        io_service as io,
        objects::{self, Key256, Kind, Object, ObjectDescriptor, ObjectListElement},
    },
};

pub struct Core {
    pub objects: objects::ObjectService,
    pub data_file: Arc<Mutex<File>>,
    pub desc_file: Arc<Mutex<File>>,
}
impl Core {
    pub fn new() -> Result<Arc<Core>, std::io::Error> {
        fs::create_dir_all(DIR_PATH).expect("Unable to create directory with data...");
        let data_file_path = io::get_data_filename("main");
        if !Path::new(&data_file_path).exists() {
            File::create(&data_file_path)?;
        }

        let data_file = OpenOptions::new()
            .read(true)
            .write(true)
            .append(true)
            .create(true)
            .open(&data_file_path)?;

        let desc_file_path = io::get_desc_filename("main");
        if !Path::new(&desc_file_path).exists() {
            File::create(&desc_file_path)?;
        }

        let desc_file = OpenOptions::new()
            .read(true)
            .write(true)
            .append(false)
            .create(true)
            .open(&desc_file_path)?;
        let mut core = Core {
            objects: objects::ObjectService::new(),
            data_file: Arc::new(Mutex::new(data_file)),
            desc_file: Arc::new(Mutex::new(desc_file)),
        };
        core.objects.load_objects_desc(Arc::clone(&core.desc_file));
        core.objects.load_objects_data(Arc::clone(&core.data_file));

        Ok(Arc::new(core))
    }

    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        return self.objects.get_data(key);
    }
    pub async fn set(&self, key: &str, kind: Kind, data: Vec<u8>) {
        let data_file = Arc::clone(&self.data_file);
        let desc_file = Arc::clone(&self.desc_file);
        let key_owned = key.to_string();
        let kind_clone = kind.clone();
        let size = data.len();
        let data_clone = data.clone();
        let mut is_existing = true;
        let mut existing_desc_offset: u64 = 0;
        match self.objects.get_desc(&key_owned.clone()) {
            None => {
                is_existing = false;
            }
            Some(desc) => existing_desc_offset = desc.desc_offset,
        }
        let obj = tokio::task::spawn_blocking(move || {
            let offset = io::save_object_in_file(&data_clone, data_file)
                .expect("Failed to write data") as u64;

            let mut desc = ObjectDescriptor {
                key: Key256::new(&key_owned),
                kind: kind_clone.clone(),
                offset,
                size: size as u64,
                is_deleted: false,
                desc_offset: 0,
            };

            let desc_data = bincode::serialize(&desc).unwrap();
            if is_existing {
                println!("edo: {}", existing_desc_offset);
                io::update_chunk_in_file(existing_desc_offset, desc_data, desc_file).unwrap();
                desc.desc_offset = existing_desc_offset;
            } else {
                let desc_offset = io::save_desc_in_file(desc_data, desc_file)
                    .expect("Failed to write descriptor");
                desc.desc_offset = desc_offset;
            }
            Object { desc, data }
        })
        .await
        .expect("spawn_blocking failed");

        self.objects.set(obj).unwrap();
    }

    pub async fn delete_soft(&self, key: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut object = self
            .objects
            .delete(key.to_string())
            .map_err(|e| format!("Failed to mark object as deleted: {}", e))?;
        object.desc.is_deleted = true;
        let data =
            bincode::serialize(&object.desc).map_err(|e| format!("Serialization error: {}", e))?;

        let desc_file: Arc<Mutex<File>> = Arc::clone(&self.desc_file);

        tokio::task::spawn_blocking(move || {
            io::update_chunk_in_file(object.desc.desc_offset, data, desc_file)
        })
        .await
        .expect("spawn_blocking failed")
        .expect("Failed to write data");
        Ok(())
    }
    pub async fn list(&self) -> Result<Vec<ObjectListElement>, Box<dyn Error + Send + Sync>> {
        return self.objects.list();
    }
    pub async fn list_by_kind(
        &self,
        kind: Kind,
    ) -> Result<Vec<ObjectListElement>, Box<dyn Error + Send + Sync>> {
        return self.objects.list_by_kind(kind);
    }
}
