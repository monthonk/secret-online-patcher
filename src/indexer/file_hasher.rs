use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};

use crate::{
    indexer::indexed_hasher::IndexedHasher,
    storage::{file_index::FileIndex, patcher_db::PatcherDatabase},
};

pub struct FileHasher {
    app_id: i64,
    db: PatcherDatabase,
}

impl FileHasher {
    pub fn new(app_id: i64, db: PatcherDatabase) -> Self {
        FileHasher { app_id, db }
    }

    pub async fn file_hash(self, file_path: &PathBuf) -> Result<IndexedHasher, anyhow::Error> {
        let mut file =
            File::open(file_path).map_err(|e| anyhow::anyhow!("Error opening file: {}", e))?;
        let metadata = file
            .metadata()
            .map_err(|e| anyhow::anyhow!("Error reading file metadata: {}", e))?;

        // TODO: better error handling for this, it should not happen
        if metadata.is_dir() {
            return Err(anyhow::anyhow!("Provided path is a directory, not a file"));
        }

        let modified_time = metadata.modified()?;
        let modified_time = DateTime::<Utc>::from(modified_time).naive_utc();
        // Check if we have a cached hash for this file
        if let Some(index) = last_index(self.app_id, file_path, &self.db).await
            && index.file_type == "FILE"
            && modified_time == index.modified_time
            && index.hash_code.is_some()
        {
            let hex_hash = index.hash_code.unwrap();
            println!(
                "hash: {}, entry: {} (cached)",
                hex_hash,
                file_path.display()
            );
            let indexed_hasher = IndexedHasher::from_hash(file_path, modified_time, &hex_hash);
            return Ok(indexed_hasher);
        }

        let mut buffer: [u8; 4096] = [0; 4096]; // Read in 4KB chunks

        let mut hasher = IndexedHasher::new(file_path, modified_time);
        while let Ok(bytes_read) = file.read(&mut buffer) {
            // Reaching end of file
            if bytes_read == 0 {
                break;
            }
            hasher.append_hash(&buffer[..bytes_read]);
        }

        let path_str = file_path.display().to_string();
        hasher.append_changed_file(&path_str);
        Ok(hasher)
    }
}

async fn last_index(app_id: i64, file_path: &Path, db: &PatcherDatabase) -> Option<FileIndex> {
    let file_path = file_path.display().to_string();
    db.get_file_index(app_id, &file_path).await.unwrap_or(None)
}
