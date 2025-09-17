use std::{fs, path::PathBuf};

use anyhow::anyhow;
use chrono::{DateTime, Utc};

use crate::{
    indexer::{file_hasher::FileHasher, indexed_hasher::IndexedHasher},
    storage::patcher_db::PatcherDatabase,
};

pub struct DirHasher {
    app_id: i64,
    db: PatcherDatabase,
}

impl DirHasher {
    pub fn new(app_id: i64, db: PatcherDatabase) -> Self {
        DirHasher { app_id, db }
    }

    pub async fn dir_hash(
        self,
        file_path: &PathBuf,
        update_index: bool,
    ) -> Result<String, anyhow::Error> {
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
        let mut hasher = IndexedHasher::new(file_path, modified_time);
        for entry_path in entries {
            let metadata = fs::metadata(&entry_path)?;
            let hex_hash = if metadata.is_dir() {
                // Recursively hash the directory
                let hasher = DirHasher::new(self.app_id, self.db.clone());
                Box::pin(hasher.dir_hash(&entry_path, update_index)).await?
            } else {
                let hasher = FileHasher::new(self.app_id, self.db.clone());
                let indexed_hasher = hasher.file_hash(&entry_path).await?;
                let file_path = indexed_hasher.file_path.display().to_string();
                let modified_time = indexed_hasher.modified_time.clone();
                let (hex_hash, _) = indexed_hasher.finalize();
                // Update index in db
                // TODO: DO NOT update if the hash is cached
                if update_index {
                    self.db
                        .upsert_file_index(
                            self.app_id,
                            &file_path,
                            "FILE",
                            &hex_hash,
                            &modified_time,
                        )
                        .await?;
                }
                hex_hash
            };
            hasher.append_hash(hex_hash.as_bytes());
        }

        let path_str = file_path.display().to_string();
        let (combined_hash, changed_file_list) = hasher.finalize();

        // List changed files,
        if !changed_file_list.is_empty() {
            println!("Changed files:");
            for changed_file in changed_file_list {
                println!(" - {}", changed_file);
            }
        }

        // Update index in db
        if update_index {
            println!("Directory modified time: {}", modified_time);

            self.db
                .upsert_file_index(
                    self.app_id,
                    &path_str,
                    "DIRECTORY",
                    &combined_hash,
                    &modified_time,
                )
                .await?;
        }

        Ok(combined_hash)
    }
}
