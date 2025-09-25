use std::{
    fs::{self, File},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use clap::{Parser, ValueEnum};
use sqlx::SqlitePool;
use zip::{ZipWriter, write::SimpleFileOptions};

use crate::{
    indexer::{
        dir_hasher::DirHasher,
        file_change::{FileChange, FileChangeType},
        indexer_config::IndexerConfig,
    },
    service::app_manager::AppManager,
    storage::{
        application_data::Application, patch_db::PatchDatabase, patcher_db::PatcherDatabase,
    },
};

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
    Update,
}

pub async fn list_apps(db: &PatcherDatabase) {
    // Implementation for listing apps
    let apps = db.list_applications().await;
    for app in apps {
        tracing::info!(
            "ID: {}, Name: {}, Version: {}, Hash: {:?}",
            app.id,
            app.name,
            app.version,
            app.hash_code
        );

        let app_id = app.id;
        let root = app.install_path.display().to_string();
        let indexed_files = db.get_files_in_directory(app_id, &root).await;
        if let Ok(files) = indexed_files {
            tracing::info!("  Indexed files for app {}:", app.name);
            for file in files {
                tracing::info!("    - {} ({})", file.file_path, file.file_type);
            }
        }
    }
}

pub async fn add_app(
    name: &str,
    version: &str,
    path: &PathBuf,
    app_manager: &AppManager,
) -> Result<(), anyhow::Error> {
    // Implementation for adding an app
    let _app = app_manager.create_application(name, version, path).await?;

    Ok(())
}

pub async fn remove_app(name: &str, db: &PatcherDatabase) {
    db.remove_application(name).await;
}

pub async fn check_app(name: &str, db: &PatcherDatabase) -> Result<(), anyhow::Error> {
    let app = db.get_application(name).await?;
    match app {
        Some(app) => {
            tracing::info!(
                "ID: {}, Name: {}, Version: {}, Hash: {:?}",
                app.id,
                app.name,
                app.version,
                app.hash_code
            );
            if app.hash_code.is_none() {
                return Err(anyhow!(
                    "Failed to check application due to missing hash code, it might not be initialized properly!"
                ));
            }
            let old_hash = app.hash_code.clone().unwrap();

            let indexer_config = IndexerConfig::new(app.id, db.clone(), false);
            let hasher = DirHasher::new(indexer_config);
            let new_hash = hasher.dir_hash(&PathBuf::from(&app.install_path)).await?;
            let (new_hash, file_changes) = new_hash.finalize().await;
            if new_hash == old_hash {
                tracing::info!("No changes detected for application {}", app.name);
            } else {
                tracing::info!("Changes detected for application {}!", app.name);
                for change in &file_changes {
                    tracing::info!(" - [{}] {}", change.change_type, change.file_path);
                }
                tracing::info!("New hash: {}", new_hash);
            }
        }
        None => {
            return Err(anyhow!("Application not found"));
        }
    }
    Ok(())
}

pub async fn update_app(
    name: &str,
    version: &str,
    db: &PatcherDatabase,
) -> Result<(), anyhow::Error> {
    let app = db.get_application(name).await?;
    match app {
        Some(app) => {
            tracing::info!(
                "ID: {}, Name: {}, Current Version: {}, Hash: {:?}",
                app.id,
                app.name,
                app.version,
                app.hash_code
            );
            if app.hash_code.is_none() {
                return Err(anyhow!(
                    "Failed to update application due to missing hash code, it might not be initialized properly!"
                ));
            }
            let old_hash = app.hash_code.clone().unwrap();

            let indexer_config = IndexerConfig::new(app.id, db.clone(), true);
            let hasher = DirHasher::new(indexer_config);
            let new_hash = hasher.dir_hash(&PathBuf::from(&app.install_path)).await?;
            let (new_hash, file_changes) = new_hash.finalize().await;
            if new_hash == old_hash {
                tracing::info!("No changes detected for application {}", app.name);
                tracing::info!("Skip updating...");
            } else {
                tracing::info!("Changes detected for application {}!", app.name);
                for change in &file_changes {
                    tracing::info!(" - [{}] {}", change.change_type, change.file_path);
                }
                tracing::info!("New hash: {}", new_hash);
                tracing::info!("Updating version to {}...", version);
                let new_version = Application {
                    id: app.id,
                    name: app.name.clone(),
                    version: version.to_string(),
                    install_path: app.install_path.clone(),
                    hash_code: Some(new_hash),
                };
                db.update_application(
                    &new_version.id,
                    &new_version.version,
                    new_version.hash_code.as_ref().unwrap(),
                )
                .await;

                // Create the zip package for the update
                // TODO: use prod directory
                let out_dir = PathBuf::from("fs_tests/patches");
                create_zip_package(&app, &version, &file_changes, &out_dir).await?;
            }
        }
        None => {
            return Err(anyhow!("Application not found"));
        }
    }
    Ok(())
}

async fn create_zip_package(
    app: &Application,
    new_version: &str,
    file_changes: &[FileChange],
    out_dir: &Path,
) -> Result<(), anyhow::Error> {
    // Create database file for the patch, this file will be added to the zip
    fs::create_dir_all(out_dir)?;
    let db_path = format!("{}/patch.db", out_dir.display());
    let _ = fs::remove_file(&db_path);
    let db_conn = format!("sqlite:{}?mode=rwc", db_path);
    let db_pool = SqlitePool::connect(&db_conn).await.unwrap();
    let patch_db = PatchDatabase::new(db_pool);
    patch_db.initialize().await;
    let patch = patch_db
        .create_patch(&app.name, &app.version, new_version)
        .await?;

    // Create zip file from the file changes
    let sanitized_app_name = app.name.replace(" ", "_");
    let zip_path = format!(
        "{}/{}_{}_update.zip",
        out_dir.display(),
        sanitized_app_name,
        new_version
    );
    let _ = fs::remove_file(&zip_path);
    let zip_file = File::create(&zip_path)?;

    let mut zip_writer = ZipWriter::new(zip_file);
    for change in file_changes {
        // Add change to the patch database
        let change_type = change.change_type.to_string().to_uppercase();
        patch_db
            .add_file_change(patch.id, &change.file_path, &change.file_type, &change_type)
            .await?;

        // Skip deleted files
        if change.change_type == FileChangeType::Deleted {
            continue;
        }

        let file_path = PathBuf::from(&change.file_path);
        let file_metadata = fs::metadata(&file_path)?;
        if file_path.is_file() {
            let options = SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(file_metadata.permissions().mode());
            zip_writer.start_file(change.file_path.clone(), options)?;
            let mut f = File::open(&file_path)?;
            std::io::copy(&mut f, &mut zip_writer)?;
        }
    }
    // Ensure the database connection is closed before finishing the zip
    drop(patch_db);
    // Then add the database file to the zip
    let mut db_file = File::open(db_path)?;
    std::io::copy(&mut db_file, &mut zip_writer)?;
    zip_writer.finish()?;
    tracing::info!("Update package created at: {}", zip_path);
    Ok(())
}
