use std::path::PathBuf;

use anyhow::anyhow;
use clap::{Parser, ValueEnum};

use crate::{
    indexer::{dir_hasher::DirHasher, indexer_config::IndexerConfig},
    service::app_manager::AppManager,
    storage::{application_data::Application, patcher_db::PatcherDatabase},
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
        println!(
            "ID: {}, Name: {}, Version: {}, Hash: {:?}",
            app.id, app.name, app.version, app.hash_code
        );
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
            println!(
                "ID: {}, Name: {}, Version: {}, Hash: {:?}",
                app.id, app.name, app.version, app.hash_code
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
            if new_hash == old_hash {
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

pub async fn update_app(
    name: &str,
    version: &str,
    db: &PatcherDatabase,
) -> Result<(), anyhow::Error> {
    let app = db.get_application(name).await?;
    match app {
        Some(app) => {
            println!(
                "ID: {}, Name: {}, Current Version: {}, Hash: {:?}",
                app.id, app.name, app.version, app.hash_code
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
            if new_hash == old_hash {
                println!("No changes detected for application {}", app.name);
                println!("Skip updating...");
            } else {
                println!("Changes detected for application {}!", app.name);
                println!("New hash: {}", new_hash);
                println!("Updating version to {}...", version);
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
            }
        }
        None => {
            return Err(anyhow!("Application not found"));
        }
    }
    Ok(())
}
