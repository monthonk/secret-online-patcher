use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use chrono::{DateTime, NaiveDateTime, Utc};

use crate::{
    indexer::{
        file_change::FileChangeType, indexed_hasher::IndexedHasher, indexer_config::IndexerConfig,
    },
    storage::db_utils,
};

pub struct FileHasher {
    config: IndexerConfig,
}

impl FileHasher {
    pub fn new(config: IndexerConfig) -> Self {
        FileHasher { config }
    }

    pub async fn file_hash(&self, file_path: &PathBuf) -> Result<IndexedHasher, anyhow::Error> {
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
        let hasher = if let Some(index) =
            db_utils::last_index(self.config.app_id, file_path, &self.config.db).await
        {
            // If the file has not been modified and we have a hash, return the cached hash
            if index.file_type == "FILE"
                && modified_time == index.modified_time
                && index.hash_code.is_some()
            {
                let hex_hash = index.hash_code.unwrap();

                IndexedHasher::from_hash(
                    file_path,
                    "FILE",
                    modified_time,
                    &hex_hash,
                    self.config.clone(),
                )
            } else {
                // Otherwise, we will recompute the hash
                let mut hasher = self.compute_file_hash(&mut file, file_path, modified_time);
                let path_str = file_path.display().to_string();
                hasher.append_changed_file(&path_str, FileChangeType::Modified);
                hasher
            }
        } else {
            // No cache entry at all, this is a new file
            let mut hasher = self.compute_file_hash(&mut file, file_path, modified_time);
            let path_str = file_path.display().to_string();
            hasher.append_changed_file(&path_str, FileChangeType::Created);
            hasher
        };

        Ok(hasher)
    }

    fn compute_file_hash(
        &self,
        file: &mut File,
        file_path: &Path,
        modified_time: NaiveDateTime,
    ) -> IndexedHasher {
        let mut buffer: [u8; 4096] = [0; 4096]; // Read in 4KB chunks

        let mut hasher = IndexedHasher::new(file_path, "FILE", modified_time, self.config.clone());
        while let Ok(bytes_read) = file.read(&mut buffer) {
            // Reaching end of file
            if bytes_read == 0 {
                break;
            }
            hasher.append_hash(&buffer[..bytes_read]);
        }

        hasher
    }
}
