use std::{fs, path::PathBuf};

use anyhow::anyhow;
use sha2::{Digest, Sha256};

use crate::indexer::file_hasher::FileHasher;

#[derive(Default)]
pub struct DirHasher {
    hasher: Sha256,
}

impl DirHasher {
    pub fn dir_hash(mut self, file_path: &PathBuf) -> Result<String, anyhow::Error> {
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
                let hasher = DirHasher::default();
                hasher.dir_hash(&entry_path)?
            } else {
                let hasher = FileHasher::default();
                hasher.file_hash(&entry_path)?
            };
            println!("hash: {}, entry: {}", hex_hash, entry_path.display());
            self.hasher.update(hex_hash.as_bytes());
        }
        let combined_hash = self.hasher.finalize();
        // Encode the hash as a hexadecimal string
        let hex_hash = base16ct::lower::encode_string(&combined_hash);
        Ok(hex_hash)
    }
}
