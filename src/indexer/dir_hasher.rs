use std::{fs, path::PathBuf};

use anyhow::anyhow;
use sha2::{Digest, Sha256};

use crate::{indexer::file_hasher::FileHasher, storage::patcher_db::PatcherDatabase};

pub struct DirHasher {
    hasher: Sha256,
    app_id: i64,
    db: PatcherDatabase,
}

impl DirHasher {
    pub fn new(app_id: i64, db: PatcherDatabase) -> Self {
        DirHasher {
            hasher: Sha256::new(),
            app_id,
            db,
        }
    }

    pub async fn dir_hash(mut self, file_path: &PathBuf) -> Result<String, anyhow::Error> {
        let mut entries = Vec::new();
        for entry in fs::read_dir(file_path)? {
            if entry.is_err() {
                return Err(anyhow!("Error reading directory"));
            }
            let entry = entry.unwrap();
            entries.push(entry.path());
        }

        // Sort the entries alphabetically by path
        entries.sort();

        // Print the sorted paths
        for entry_path in entries {
            let metadata = fs::metadata(&entry_path)?;
            let hex_hash = if metadata.is_dir() {
                // Recursively hash the directory
                let hasher = DirHasher::new(self.app_id, self.db.clone());
                Box::pin(hasher.dir_hash(&entry_path)).await?
            } else {
                let hasher = FileHasher::new(self.app_id, self.db.clone());
                hasher.file_hash(&entry_path).await?
            };
            self.hasher.update(hex_hash.as_bytes());
        }
        let combined_hash = self.hasher.finalize();
        // Encode the hash as a hexadecimal string
        let hex_hash = base16ct::lower::encode_string(&combined_hash);
        println!("hash: {}, entry: {}", hex_hash, file_path.display());
        Ok(hex_hash)
    }
}
