use clap::{Parser, ValueEnum};

use crate::storage::patcher_db::PatcherDatabase;

#[derive(Parser, Debug)]
pub struct Args {
    /// Operation to perform
    #[arg(help = "Operation to perform")]
    pub op: Operation,
}

#[derive(ValueEnum, Clone, Debug)]
#[clap(rename_all = "kebab_case")]
pub enum Operation {
    List,
    AddApp,
}

pub async fn list_apps(db: &PatcherDatabase) {
    // Mock data
    let mock_apps = vec![
        ("App1", "1.0.0", "hashcode1"),
        ("App2", "2.3.4", "hashcode2"),
    ];
    for app in mock_apps {
        db.add_application(app.0, app.1, app.2).await;
    }

    // Implementation for listing apps
    let apps = db.list_applications().await;
    for app in apps {
        println!(
            "ID: {}, Name: {}, Version: {}, Hash: {}",
            app.id, app.name, app.version, app.hash_code
        );
    }
}

pub fn add_app() {
    // Implementation for adding an app
    println!("Adding a new application...");
}
