use crate::storage::patcher_db::PatcherDatabase;

#[derive(Clone)]
pub struct IndexerConfig {
    pub app_id: i64,
    pub db: PatcherDatabase,
    pub update_index: bool,
}

impl IndexerConfig {
    pub fn new(app_id: i64, db: PatcherDatabase, update_index: bool) -> Self {
        IndexerConfig {
            app_id,
            db,
            update_index,
        }
    }
}
