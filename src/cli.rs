use clap::{Parser, ValueEnum};

use crate::storage::{application_data::Application, patcher_db::PatcherDatabase};

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
    pub app_path: Option<String>,
}

#[derive(ValueEnum, Clone, Debug)]
#[clap(rename_all = "kebab_case")]
pub enum Operation {
    List,
    AddApp,
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

pub async fn add_app(app: Application, db: &PatcherDatabase) {
    // Implementation for adding an app
    db.add_application(&app.name, &app.version, &app.hash_code)
        .await;
}
