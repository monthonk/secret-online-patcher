use secret_online_patcher::storage::patcher_db::PatcherDatabase;

pub async fn verify_index(
    app_id: i64,
    file_path: &str,
    should_exist: bool,
    expected_hash: Option<&str>,
    db: &PatcherDatabase,
) {
    let index = db
        .get_file_index(app_id, file_path)
        .await
        .expect("failed to get file index");
    if should_exist {
        assert!(index.is_some(), "file index not found");
        let index = index.unwrap();
        if let Some(expected_hash) = expected_hash {
            assert_eq!(index.hash_code.unwrap(), expected_hash);
        }
    } else {
        assert!(index.is_none(), "file index should not exist");
    }
}
