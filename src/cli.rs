use std::{fs, path::PathBuf};

use anyhow::anyhow;
use clap::{Parser, ValueEnum};
use sha2::{Digest, Sha256};

use crate::{indexer::file_hasher::FileHasher, storage::patcher_db::PatcherDatabase};

#[derive(Parser, Debug)]
pub struct Args {
    /// Operation to perform
    #[arg(help = "Operation to perform")]
    pub op: Operation,

    // Application name to add
    #[arg(
        long,
        help = "Name of the application to add, required when operation is add-app"
    )]
    pub app_name: Option<String>,

    #[arg(
        long,
        help = "Version of the application to add, required when operation is add-app"
    )]
    pub app_version: Option<String>,

    #[arg(
        long,
        help = "Path to the application to add, required when operation is add-app"
    )]
    pub app_path: Option<PathBuf>,
}

#[derive(ValueEnum, Clone, Debug)]
#[clap(rename_all = "kebab_case")]
pub enum Operation {
    List,
    AddApp,
    RemoveApp,
}

pub async fn list_apps(db: &PatcherDatabase) {
    // Implementation for listing apps
    let apps = db.list_applications().await;
    for app in apps {
        println!(
            "ID: {}, Name: {}, Version: {}, Hash: {}",
            app.id, app.name, app.version, app.hash_code
        );
    }
}

pub async fn add_app(
    name: &str,
    version: &str,
    path: &PathBuf,
    db: &PatcherDatabase,
) -> Result<(), anyhow::Error> {
    // Compute hash code for the app
    // Hash code is the CRC32 hash of the hash from all files in the app directory
    // order by their names.
    let mut entries = Vec::new();
    for entry in fs::read_dir(path)? {
        if entry.is_err() {
            return Err(anyhow!("Error reading directory"));
        }
        let entry = entry.unwrap();
        entries.push(entry.path());
    }

    // Sort the entries alphabetically by path
    entries.sort();

    let mut combined_hash = Sha256::new();
    // Print the sorted paths
    for entry_path in entries {
        let hasher = FileHasher::default();
        let hex_hash = hasher.file_hash(&entry_path)?;
        println!("hash: {}, file: {}", hex_hash, entry_path.display());
        combined_hash.update(hex_hash.as_bytes());
    }
    let combined_hash = combined_hash.finalize();
    let app_hash = base16ct::lower::encode_string(&combined_hash);
    println!("Application hash is {}", app_hash);

    // Implementation for adding an app
    db.add_application(name, version, &app_hash, path).await;

    Ok(())
}

pub async fn remove_app(name: &str, db: &PatcherDatabase) {
    db.remove_application(name).await;
}
