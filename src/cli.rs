use std::path::PathBuf;

use anyhow::anyhow;
use clap::{Parser, ValueEnum};

use crate::{indexer::dir_hasher::DirHasher, storage::patcher_db::PatcherDatabase};

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
    Check,
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
    let hasher = DirHasher::default();
    let app_hash = hasher.dir_hash(path)?;
    println!("Application hash is {}", app_hash);

    // Implementation for adding an app
    db.add_application(name, version, &app_hash, path).await;

    Ok(())
}

pub async fn remove_app(name: &str, db: &PatcherDatabase) {
    db.remove_application(name).await;
}

pub async fn check_app(name: &str, db: &PatcherDatabase) -> Result<(), anyhow::Error> {
    let app = db.get_application(name).await;
    match app {
        Some(app) => {
            println!(
                "ID: {}, Name: {}, Version: {}, Hash: {}",
                app.id, app.name, app.version, app.hash_code
            );
            let hasher = DirHasher::default();
            let new_hash = hasher.dir_hash(&PathBuf::from(&app.install_path))?;
            if new_hash == app.hash_code {
                println!("No changes detected for application {}", app.name);
            } else {
                println!("Changes detected for application {}!", app.name);
                println!("New hash: {}", new_hash);
            }
        }
        None => {
            return Err(anyhow!("Application not found"));
        }
    }
    Ok(())
}
