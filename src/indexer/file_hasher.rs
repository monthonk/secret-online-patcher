use std::{fs::File, io::Read, path::PathBuf};

use sha2::{Digest, Sha256};

pub struct FileHasher {
    hasher: Sha256,
}

impl FileHasher {
    pub fn new() -> Self {
        Self {
            hasher: Sha256::new(),
        }
    }

    pub fn file_hash(mut self, file_path: &PathBuf) -> Result<String, anyhow::Error> {
        let mut file =
            File::open(file_path).map_err(|e| anyhow::anyhow!("Error opening file: {}", e))?;
        // TODO: Support hashing directories with recursive file hashing
        if file
            .metadata()
            .map_err(|e| anyhow::anyhow!("Error reading file metadata: {}", e))?
            .is_dir()
        {
            return Err(anyhow::anyhow!("Provided path is a directory, not a file"));
        }
        let mut buffer: [u8; 4096] = [0; 4096]; // Read in 4KB chunks

        while let Ok(bytes_read) = file.read(&mut buffer) {
            // Reaching end of file
            if bytes_read == 0 {
                break;
            }
            // self.hasher.update(&buffer[..bytes_read]);
            self.hasher.update(&buffer[..bytes_read]);
        }
        let hash = self.hasher.finalize();
        // Encode the hash as a hexadecimal string
        let hex_hash = base16ct::lower::encode_string(&hash);
        Ok(hex_hash)
    }
}
