use clap::Parser;
use secret_online_patcher::{
    cli::{Args, Operation},
    storage::patcher_db::PatcherDatabase,
};
use sqlx::SqlitePool;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Initialize database connection
    let db_pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    let patcher_db = PatcherDatabase::new(db_pool);
    patcher_db.initialize().await;

    match args.op {
        Operation::List => {
            // Call the function to list applications
            secret_online_patcher::cli::list_apps(&patcher_db).await;
        }
        Operation::AddApp => {
            // Call the function to add an app
            secret_online_patcher::cli::add_app();
        }
    }
}
