use std::{fs, path::Path};

use secret_online_patcher::indexer::{
    file_change::FileChangeType, file_hasher::FileHasher, indexer_config::IndexerConfig,
};
use sqlx::SqlitePool;

use crate::common::{
    db_util::verify_index,
    test_util::{initialize_test_app, initialize_test_db, initialize_test_dir},
};

#[sqlx::test]
async fn file_hasher_with_new_file(db_pool: SqlitePool) {
    let test_dir = initialize_test_dir("file_hasher_with_new_file");
    let db = initialize_test_db(&db_pool).await;
    let app = initialize_test_app(&test_dir, &db).await;

    // Create a test file
    let test_file = format!("{}/test_file.txt", test_dir);
    fs::write(&test_file, "Hello, world!").unwrap();

    let config = IndexerConfig::new(app.id, db.clone(), true);
    let file_hasher = FileHasher::new(config);
    let hash_result = file_hasher
        .file_hash(&Path::new(&test_file).to_path_buf())
        .await
        .expect("failed to hash file");
    let (hex_hash, changed_files) = hash_result.finalize().await;
    assert_eq!(hex_hash.len(), 64); // SHA-256 hash length in hex
    assert_eq!(
        hex_hash,
        "315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3"
    );
    assert_eq!(changed_files.len(), 1);
    assert_eq!(changed_files[0].change_type, FileChangeType::Created);
    assert_eq!(changed_files[0].file_path, test_file);

    // Verify data in the database
    verify_index(app.id, &test_file, true, Some(&hex_hash), &db).await;
}

#[sqlx::test]
async fn file_hasher_with_modified_file(db_pool: SqlitePool) {
    let test_dir = initialize_test_dir("file_hasher_with_modified_file");
    let db = initialize_test_db(&db_pool).await;
    let app = initialize_test_app(&test_dir, &db).await;

    // Create a test file
    let test_file = format!("{}/test_file.txt", test_dir);
    fs::write(&test_file, "Hello, world!").unwrap();

    let config = IndexerConfig::new(app.id, db.clone(), true);
    let file_hasher = FileHasher::new(config);
    let hash_result = file_hasher
        .file_hash(&Path::new(&test_file).to_path_buf())
        .await
        .expect("failed to hash file");
    let (hex_hash, changed_files) = hash_result.finalize().await;
    assert_eq!(hex_hash.len(), 64); // SHA-256 hash length in hex
    assert_eq!(
        hex_hash,
        "315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3"
    );
    assert_eq!(changed_files.len(), 1);
    assert_eq!(changed_files[0].change_type, FileChangeType::Created);
    assert_eq!(changed_files[0].file_path, test_file);

    // Now modify the file
    fs::write(&test_file, "Hello, Rust!").unwrap();

    // Verify that the hash changes
    let hash_result = file_hasher
        .file_hash(&Path::new(&test_file).to_path_buf())
        .await
        .expect("failed to hash file");
    let (hex_hash, changed_files) = hash_result.finalize().await;
    assert_eq!(hex_hash.len(), 64); // SHA-256 hash length in hex
    assert_eq!(
        hex_hash,
        "12a967da1e8654e129d41e3c016f14e81e751e073feb383125bf82080256ca19"
    );
    assert_eq!(changed_files.len(), 1);
    assert_eq!(changed_files[0].change_type, FileChangeType::Modified);
    assert_eq!(changed_files[0].file_path, test_file);

    // Verify data in the database
    verify_index(app.id, &test_file, true, Some(&hex_hash), &db).await;
}

#[sqlx::test]
async fn file_hasher_fail_with_non_existing_file(db_pool: SqlitePool) {
    let test_dir = initialize_test_dir("file_hasher_fail_with_non_existing_file");
    let db = initialize_test_db(&db_pool).await;
    let app = initialize_test_app(&test_dir, &db).await;

    // Test with a non-existing file
    let test_file = format!("{}/non_existing.txt", &test_dir);

    let config = IndexerConfig::new(app.id, db.clone(), true);
    let file_hasher = FileHasher::new(config);
    let hash_result = file_hasher
        .file_hash(&Path::new(&test_file).to_path_buf())
        .await;
    assert!(hash_result.is_err());
    assert!(
        hash_result
            .err()
            .unwrap()
            .to_string()
            .contains("Error opening file")
    );

    // Verify data in the database
    verify_index(app.id, &test_file, false, None, &db).await;
}
