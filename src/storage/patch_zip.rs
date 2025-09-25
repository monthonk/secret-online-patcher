use std::{
    fs::{self, File},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use sqlx::SqlitePool;
use zip::{ZipWriter, write::SimpleFileOptions};

use crate::{
    indexer::file_change::{FileChange, FileChangeType},
    storage::{application_data::Application, patch_db::PatchDatabase},
};

pub struct PatchZip {
    // ID of the patch in the database, None if not initialized
    pub patch_id: Option<i64>,
    pub app: Application,
    pub out_dir: PathBuf,
    pub db: PatchDatabase,
    // Zip writer for creating the patch zip file, None if not initialized
    pub zip_writer: Option<ZipWriter<File>>,
}

impl PatchZip {
    pub fn new(out_dir: &Path, app: &Application) -> Self {
        // Create database file for the patch, this file will be added to the zip
        let db_path = format!("{}/patch.db", out_dir.display());
        // Make sure to remove any existing database file
        let _ = fs::remove_file(&db_path);
        let db_conn = format!("sqlite:{}?mode=rwc", db_path);
        let db_pool = futures::executor::block_on(SqlitePool::connect(&db_conn)).unwrap();
        let patch_db = PatchDatabase::new(db_pool);
        futures::executor::block_on(patch_db.initialize());
        PatchZip {
            app: app.clone(),
            patch_id: None,
            out_dir: out_dir.to_path_buf(),
            db: patch_db,
            zip_writer: None,
        }
    }

    pub async fn initialize_patch(&mut self, new_version: &str) -> Result<i64, anyhow::Error> {
        // If already initialized, return the existing patch ID
        if let Some(patch_id) = self.patch_id {
            return Ok(patch_id);
        }

        let app_name = &self.app.name;
        let old_version = &self.app.version;
        let patch = self
            .db
            .create_patch(app_name, old_version, new_version)
            .await?;

        // Create zip file for the changes
        let sanitized_app_name = app_name.replace(" ", "_");
        let package_name = format!("{}_{}_update", sanitized_app_name, new_version);
        let zip_path = format!("{}/{}.zip", self.out_dir.display(), package_name);
        let _ = fs::remove_file(&zip_path);
        let zip_file = File::create(&zip_path)?;

        self.zip_writer = Some(ZipWriter::new(zip_file));
        self.patch_id = Some(patch.id);
        Ok(patch.id)
    }

    pub async fn append_changed_file(&mut self, change: &FileChange) -> Result<(), anyhow::Error> {
        if self.patch_id.is_none() || self.zip_writer.is_none() {
            return Err(anyhow::anyhow!("PatchZip not initialized"));
        }

        let patch_id = self.patch_id.unwrap();
        let mut zip_writer = self.zip_writer.as_mut().unwrap();
        let change_type = change.change_type.to_string().to_uppercase();
        self.db
            .add_file_change(patch_id, &change.file_path, &change.file_type, &change_type)
            .await?;

        // Skip deleted files
        if change.change_type == FileChangeType::Deleted {
            return Ok(());
        }

        let file_path = PathBuf::from(&change.file_path);
        let file_metadata = fs::metadata(&file_path)?;
        if file_path.is_file() {
            let options = SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(file_metadata.permissions().mode());
            let trimmed_path = file_path.strip_prefix(&self.app.install_path)?;
            let path_in_zip = format!("{}/{}", self.app.name, trimmed_path.display());
            zip_writer.start_file(path_in_zip, options)?;
            let mut f = File::open(&file_path)?;
            std::io::copy(&mut f, &mut zip_writer)?;
        }
        Ok(())
    }

    pub async fn finalize(mut self) -> Result<(), anyhow::Error> {
        if self.patch_id.is_none() || self.zip_writer.is_none() {
            return Err(anyhow::anyhow!("PatchZip not initialized"));
        }
        let mut zip_writer = self.zip_writer.take().unwrap();

        // Ensure the database connection is closed before finishing the zip
        drop(self.db);
        // Then add the database file to the zip
        zip_writer.start_file(
            format!("{}/patch.db", self.app.name),
            SimpleFileOptions::default(),
        )?;

        let db_path = format!("{}/patch.db", self.out_dir.display());
        let mut db_file = File::open(db_path)?;
        std::io::copy(&mut db_file, &mut zip_writer)?;
        zip_writer.finish()?;
        tracing::info!(
            "Update package created successfully at {}",
            self.out_dir.display()
        );
        Ok(())
    }
}
