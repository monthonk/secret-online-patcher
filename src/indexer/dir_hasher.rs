use std::{collections::HashMap, fs, path::PathBuf};

use anyhow::anyhow;
use chrono::{DateTime, Utc};

use crate::{
    indexer::{
        file_change::FileChangeType, file_hasher::FileHasher, file_info::FileInfo,
        indexed_hasher::IndexedHasher, indexer_config::IndexerConfig,
    },
    storage::{db_utils, patcher_db::PatcherDatabase},
};

pub struct DirHasher {
    config: IndexerConfig,
}

impl DirHasher {
    pub fn new(config: IndexerConfig) -> Self {
        DirHasher { config }
    }

    pub async fn dir_hash(&self, file_path: &PathBuf) -> Result<IndexedHasher, anyhow::Error> {
        let mut entries = Vec::new();
        let metadata = fs::metadata(file_path)?;
        if !metadata.is_dir() {
            return Err(anyhow!("Provided path is not a directory"));
        }
        let modified_time = metadata.modified()?;
        let modified_time = DateTime::<Utc>::from(modified_time).naive_utc();

        // Check if we have a cached hash for this directory to see if any files are deleted
        let last_index = db_utils::last_index(self.config.app_id, file_path, &self.config.db).await;
        let mut previous_children = HashMap::new();
        if let Some(_index) = &last_index {
            let previous_files =
                db_utils::list_indexed_files(self.config.app_id, file_path, true, &self.config.db)
                    .await?;
            for file in previous_files {
                previous_children.insert(
                    file.file_path.clone(),
                    FileInfo::new(&file.file_path, &file.file_type),
                );
            }
        }

        for entry in fs::read_dir(file_path)? {
            if entry.is_err() {
                return Err(anyhow!("Error reading directory"));
            }
            let entry = entry.unwrap();
            entries.push(entry.path());
        }

        // Sort the entries alphabetically by path
        entries.sort();

        // Keep track of current children to detect deletions
        let mut current_children = HashMap::new();

        // Recompute the hash by combining the hashes of all entries
        let mut dir_hasher =
            IndexedHasher::new(file_path, "DIRECTORY", modified_time, self.config.clone());
        for entry_path in &entries {
            let path_str = entry_path.display().to_string();
            let metadata = fs::metadata(entry_path)?;

            // Find the last index entry for this path, if any
            let last_entry =
                db_utils::last_index(self.config.app_id, entry_path, &self.config.db).await;
            if metadata.is_dir() {
                // Add to current children
                current_children.insert(path_str.clone(), FileInfo::new(&path_str, "DIRECTORY"));

                if last_entry.is_none() {
                    // New directory
                    dir_hasher.append_changed_file(&path_str, FileChangeType::Created);
                }

                // Recursively hash the directory
                let hasher = DirHasher::new(self.config.clone());
                let result = Box::pin(hasher.dir_hash(entry_path)).await?;
                let hex_hash = dir_hasher.extend(result).await;
                if let Some(entry) = last_entry
                    && entry.hash_code != Some(hex_hash)
                {
                    // Directory modified
                    dir_hasher.append_changed_file(&path_str, FileChangeType::Modified);
                }
            } else {
                // Add to current children
                current_children.insert(path_str.clone(), FileInfo::new(&path_str, "FILE"));

                // Hash the file
                let hasher = FileHasher::new(self.config.clone());
                let result = hasher.file_hash(entry_path).await?;
                dir_hasher.extend(result).await;
            };
        }

        // Find deleted files and directories
        for (file_path, file_info) in previous_children {
            if !current_children.contains_key(&file_path) {
                // If a directory was deleted, we need to mark all its children as deleted too
                if file_info.file_type == "DIRECTORY" {
                    let previous_files = db_utils::list_indexed_files(
                        self.config.app_id,
                        &PathBuf::from(&file_path),
                        false,
                        &self.config.db,
                    )
                    .await?;
                    for file in previous_files {
                        // Also delete it from the database if needed
                        if self.config.update_index {
                            delete_file_index(self.config.app_id, &file.file_path, &self.config.db)
                                .await?;
                        }
                        dir_hasher.append_changed_file(file.file_path, FileChangeType::Deleted);
                    }
                }

                // Also delete it from the database if needed
                if self.config.update_index {
                    delete_file_index(self.config.app_id, &file_path, &self.config.db).await?;
                }
                dir_hasher.append_changed_file(file_path, FileChangeType::Deleted);
            }
        }
        Ok(dir_hasher)
    }
}

async fn delete_file_index(
    app_id: i64,
    file_path: &str,
    db: &PatcherDatabase,
) -> Result<bool, anyhow::Error> {
    db.delete_file_index(app_id, file_path)
        .await
        .map_err(|e| anyhow::anyhow!("Error deleting old file index entry: {}", e))
}
