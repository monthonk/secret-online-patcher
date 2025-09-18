use std::{fs, path::PathBuf};

use anyhow::anyhow;
use chrono::{DateTime, Utc};

use crate::indexer::{
    file_hasher::FileHasher, indexed_hasher::IndexedHasher, indexer_config::IndexerConfig,
};

pub struct DirHasher {
    config: IndexerConfig,
}

impl DirHasher {
    pub fn new(config: IndexerConfig) -> Self {
        DirHasher { config }
    }

    pub async fn dir_hash(self, file_path: &PathBuf) -> Result<IndexedHasher, anyhow::Error> {
        let mut entries = Vec::new();
        let metadata = fs::metadata(file_path)?;
        if !metadata.is_dir() {
            return Err(anyhow!("Provided path is not a directory"));
        }
        let modified_time = metadata.modified()?;
        let modified_time = DateTime::<Utc>::from(modified_time).naive_utc();

        for entry in fs::read_dir(file_path)? {
            if entry.is_err() {
                return Err(anyhow!("Error reading directory"));
            }
            let entry = entry.unwrap();
            entries.push(entry.path());
        }

        // Sort the entries alphabetically by path
        entries.sort();

        // Recompute the hash by combining the hashes of all entries
        let mut dir_hasher =
            IndexedHasher::new(file_path, "DIRECTORY", modified_time, self.config.clone());
        for entry_path in entries {
            let metadata = fs::metadata(&entry_path)?;
            if metadata.is_dir() {
                // Recursively hash the directory
                let hasher = DirHasher::new(self.config.clone());
                let result = Box::pin(hasher.dir_hash(&entry_path)).await?;
                dir_hasher.extend(result).await;
            } else {
                let hasher = FileHasher::new(self.config.clone());
                let result = hasher.file_hash(&entry_path).await?;
                dir_hasher.extend(result).await;
            };
        }

        Ok(dir_hasher)
    }
}
