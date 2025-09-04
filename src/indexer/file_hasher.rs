use std::{fs::File, io::Read, path::PathBuf};

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

use crate::storage::{file_index::FileIndex, patcher_db::PatcherDatabase};

pub struct FileHasher {
    hasher: Sha256,
    app_id: i64,
    db: PatcherDatabase,
}

impl FileHasher {
    pub fn new(app_id: i64, db: PatcherDatabase) -> Self {
        FileHasher {
            hasher: Sha256::new(),
            app_id,
            db,
        }
    }

    pub async fn file_hash(mut self, file_path: &PathBuf) -> Result<String, anyhow::Error> {
        let mut file =
            File::open(file_path).map_err(|e| anyhow::anyhow!("Error opening file: {}", e))?;
        let metadata = file
            .metadata()
            .map_err(|e| anyhow::anyhow!("Error reading file metadata: {}", e))?;

        // TODO: better error handling for this, it should not happen
        if metadata.is_dir() {
            return Err(anyhow::anyhow!("Provided path is a directory, not a file"));
        }

        let system_time = metadata.modified()?;
        let current_modified_time = DateTime::<Utc>::from(system_time).naive_utc();
        if let Some(index) = last_index(self.app_id, file_path, &self.db).await {
            if index.file_type == "FILE"
                && current_modified_time == index.modified_time
                && index.hash_code.is_some()
            {
                let hex_hash = index.hash_code.unwrap();
                println!(
                    "hash: {}, entry: {} (cached)",
                    hex_hash,
                    file_path.display()
                );
                return Ok(hex_hash);
            }
        }

        let mut buffer: [u8; 4096] = [0; 4096]; // Read in 4KB chunks

        while let Ok(bytes_read) = file.read(&mut buffer) {
            // Reaching end of file
            if bytes_read == 0 {
                break;
            }
            self.hasher.update(&buffer[..bytes_read]);
        }
        let hash = self.hasher.finalize();
        // Encode the hash as a hexadecimal string
        let hex_hash = base16ct::lower::encode_string(&hash);
        println!(
            "hash: {}, entry: {} (recomputed)",
            hex_hash,
            file_path.display()
        );
        Ok(hex_hash)
    }
}

async fn last_index(app_id: i64, file_path: &PathBuf, db: &PatcherDatabase) -> Option<FileIndex> {
    let file_path = file_path.display().to_string();
    db.get_file_index(app_id, &file_path).await.unwrap_or(None)
}
