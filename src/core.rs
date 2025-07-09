use std::{
    error::Error,
    fs::{self, File, OpenOptions},
    io::Write,
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{
    DIR_PATH, io_service as io,
    object_service::{self, Key256, Kind, Object, ObjectDescriptor, ObjectListElement},
};

struct SetMessage {
    key: String,
    kind: Kind,
    data: Vec<u8>,
}

pub struct Core {
    pub objects: object_service::ObjectService,
    pub data_file: Arc<Mutex<File>>,
    pub desc_file: Arc<Mutex<File>>,
    set_tx: UnboundedSender<SetMessage>,
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
            .append(true)
            .create(true)
            .open(&desc_file_path)?;

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<SetMessage>();

        let mut core = Core {
            objects: object_service::ObjectService::new(),
            data_file: Arc::new(Mutex::new(data_file)),
            desc_file: Arc::new(Mutex::new(desc_file)),
            set_tx: tx,
        };
        core.objects.load_objects_desc(Arc::clone(&core.desc_file));
        core.objects.load_objects_data(Arc::clone(&core.data_file));
        let core_arc = Arc::new(core);
        spawn_set_task(Arc::clone(&core_arc), rx);
        Ok(core_arc)
    }
    pub async fn shutdown(&self) {
        println!("Exit program. Starting cleaning");
        drop(self.set_tx.clone());
        // wait for finish io task blocking
        tokio::time::sleep(Duration::from_millis(500)).await;

        let mut desc_file = self.desc_file.lock().unwrap();
        desc_file.flush().unwrap();
        let mut data_file = self.data_file.lock().unwrap();
        data_file.flush().unwrap();
        println!("Cleaning ends up. Closing...");
    }

    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        return self.objects.get(key);
    }
    pub async fn set(&self, key: &str, kind: Kind, data: Vec<u8>) {
        let size = data.clone().len();
        match self.set_tx.send(SetMessage {
            key: key.to_string(),
            kind: kind,
            data,
        }) {
            Err(_) => {}
            Ok(_) => {
                let start: std::time::Instant = std::time::Instant::now();

                let duration = start.elapsed();
                println!("SET completed in {:.2?} ({} bytes)", duration, size);
            }
        }
    }
    pub async fn delete_soft(&self, key: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let object = self
            .objects
            .delete(key.to_string())
            .map_err(|e| format!("Failed to mark object as deleted: {}", e))?;

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

fn spawn_set_task(core: Arc<Core>, mut set_rx: UnboundedReceiver<SetMessage>) {
    tokio::task::spawn_blocking(move || {
        while let Some(SetMessage { key, kind, data }) = set_rx.blocking_recv() {
            let size = data.len();

            let data_file = Arc::clone(&core.data_file);
            let data_clone = data.clone();

            let offset = io::save_object_in_file(&data_clone, data_file)
                .expect("Failed to write data") as u64;

            let desc = ObjectDescriptor {
                key: Key256::new(&key),
                kind: kind.clone(),
                offset,
                size: size as u64,
                is_deleted: false,
                is_mem_store: true,
                desc_offset: 0,
            };

            let desc_data = bincode::serialize(&desc).unwrap();
            let desc_file = Arc::clone(&core.desc_file);

            let desc_offset =
                io::save_desc_in_file(desc_data, desc_file).expect("Failed to write descriptor");

            println!("{}", desc_offset);

            let obj = Object {
                desc: ObjectDescriptor {
                    key: Key256::new(&key),
                    kind,
                    offset,
                    size: size as u64,
                    is_deleted: false,
                    is_mem_store: true,
                    desc_offset,
                },
                data,
            };
            core.objects.set(obj).unwrap();
        }
        println!("Set task exiting – channel closed");
    });
}
