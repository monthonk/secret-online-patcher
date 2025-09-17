use std::path::PathBuf;

use chrono::NaiveDateTime;
use sha2::{Digest, Sha256};

// A struct representing a hasher with a list of indexed file paths.
// If a previous index exists, the list will contain only paths that have changed since the last index.
pub struct IndexedHasher {
    // TODO: move file info to a new struct
    pub file_path: PathBuf,
    pub modified_time: NaiveDateTime,
    pub hasher: Sha256,
    pub cached_hash: Option<String>,
    pub changed_files: Vec<String>,
}

impl IndexedHasher {
    pub fn new(file_path: &PathBuf, modified_time: NaiveDateTime) -> Self {
        IndexedHasher {
            file_path: file_path.clone(),
            modified_time,
            hasher: Sha256::new(),
            cached_hash: None,
            changed_files: Vec::new(),
        }
    }

    pub fn from_hash(
        file_path: &PathBuf,
        modified_time: NaiveDateTime,
        hex_hash: impl AsRef<str>,
    ) -> Self {
        let hasher = Sha256::new();
        IndexedHasher {
            file_path: file_path.clone(),
            modified_time,
            hasher,
            cached_hash: Some(hex_hash.as_ref().to_string()),
            changed_files: Vec::new(),
        }
    }

    /// Append a hexadecimal hash string to the current hash without adding the file path to the changed files list.
    pub fn append_hash(&mut self, data: impl AsRef<[u8]>) {
        self.hasher.update(data);
    }

    /// Append a changed file path to the list of changed files without updating the hash.
    pub fn append_changed_file(&mut self, file_path: impl AsRef<str>) {
        self.changed_files.push(file_path.as_ref().to_string());
    }

    /// Extend the list of changed files with another IndexedHasher's changed files
    /// and combine their hashes.
    ///
    /// The provided IndexedHasher is consumed in the process.
    pub fn extend(&mut self, other: IndexedHasher) {
        let (hex_hash, changed_files) = other.finalize();

        self.hasher.update(hex_hash.as_bytes());
        self.changed_files.extend(changed_files);
    }

    pub fn finalize(self) -> (String, Vec<String>) {
        let path_str = self.file_path.display().to_string();
        if let Some(cached_hash) = self.cached_hash {
            println!("hash: {}, entry: {} (cached)", cached_hash, path_str);
            // If we have a cached hash, return it directly without recomputing,
            // and return an empty list of changed files.
            return (cached_hash, Vec::new());
        }

        let hash = self.hasher.finalize();
        // Encode the hash as a hexadecimal string
        let hex_hash = base16ct::lower::encode_string(&hash);
        println!("hash: {}, entry: {} (recomputed)", hex_hash, &path_str);
        (hex_hash, self.changed_files)
    }
}
