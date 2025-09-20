use std::{fs, path::Path};

use secret_online_patcher::indexer::{
    dir_hasher::DirHasher,
    file_change::{FileChange, FileChangeType},
    indexer_config::IndexerConfig,
};
use sqlx::SqlitePool;

use crate::common::{
    db_util::verify_index,
    test_util::{initialize_test_app, initialize_test_db, initialize_test_dir},
};

fn verify_change(expected_file: &str, expected_type: FileChangeType, changed_files: &[FileChange]) {
    let changed_file = changed_files.iter().find(|f| f.file_path == expected_file);
    assert!(changed_file.is_some());
    assert_eq!(changed_file.unwrap().change_type, expected_type);
}

#[sqlx::test]
async fn dir_hasher_with_new_dir(db_pool: SqlitePool) {
    let test_dir = initialize_test_dir("dir_hasher_with_new_dir");
    let db = initialize_test_db(&db_pool).await;
    let app = initialize_test_app(&test_dir, &db).await;

    // Create a sub-directory with some files
    let outer_file = format!("{}/outer_file1.txt", test_dir);
    let sub_dir = format!("{}/subdir", test_dir);
    let inner_file1 = format!("{}/inner_file1.txt", sub_dir);
    let inner_file2 = format!("{}/inner_file2.txt", sub_dir);
    fs::write(&outer_file, "Outer file 1 content").unwrap();
    fs::create_dir_all(&sub_dir).unwrap();
    fs::write(&inner_file1, "Inner file 1 content").unwrap();
    fs::write(&inner_file2, "Inner file 2 content").unwrap();

    let config = IndexerConfig::new(app.id, db.clone(), true);
    let dir_hasher = DirHasher::new(config);
    let hash_result = dir_hasher
        .dir_hash(&Path::new(&test_dir).to_path_buf())
        .await
        .expect("failed to hash directory");
    let (hex_hash, changed_files) = hash_result.finalize().await;
    assert_eq!(hex_hash.len(), 64); // SHA-256 hash length in hex
    assert_eq!(
        hex_hash,
        "2ab14938127707cd534778654ef4d4400f9e26571acfe316074ead23155c734b"
    );
    assert_eq!(changed_files.len(), 4);
    verify_change(&outer_file, FileChangeType::Created, &changed_files);
    verify_change(&sub_dir, FileChangeType::Created, &changed_files);
    verify_change(&inner_file1, FileChangeType::Created, &changed_files);
    verify_change(&inner_file2, FileChangeType::Created, &changed_files);

    // Verify data in the database
    verify_index(app.id, &test_dir, true, Some(&hex_hash), &db).await;
    verify_index(
        app.id,
        &outer_file,
        true,
        Some("9058c9405a63ce79c2235326d65e409b12026f72e41b488af2af6b1020f51c85"),
        &db,
    )
    .await;
    verify_index(
        app.id,
        &sub_dir,
        true,
        Some("b8bdf07b28bdfe1bb646d7680c762988efa523e1bad0f442b1dda1f11ca4b405"),
        &db,
    )
    .await;
    verify_index(
        app.id,
        &inner_file1,
        true,
        Some("eadae08b8cab3b95a3458a662af5591d314bd4e4525a7b5d6381aa56b5eda191"),
        &db,
    )
    .await;
    verify_index(
        app.id,
        &inner_file2,
        true,
        Some("7685580f5e71563c3d1831f9fe1d4da6f4ee42e76b3bdfb1b90d84a9bb739744"),
        &db,
    )
    .await;
}

#[sqlx::test]
async fn dir_hasher_with_modified_dir(db_pool: SqlitePool) {
    let test_dir = initialize_test_dir("dir_hasher_with_modified_dir");
    let db = initialize_test_db(&db_pool).await;
    let app = initialize_test_app(&test_dir, &db).await;

    // Create a sub-directory with some files
    let outer_file = format!("{}/outer_file1.txt", test_dir);
    let sub_dir = format!("{}/subdir", test_dir);
    let inner_file1 = format!("{}/inner_file1.txt", sub_dir);
    let inner_file2 = format!("{}/inner_file2.txt", sub_dir);
    fs::write(&outer_file, "Outer file 1 content").unwrap();
    fs::create_dir_all(&sub_dir).unwrap();
    fs::write(&inner_file1, "Inner file 1 content").unwrap();
    fs::write(&inner_file2, "Inner file 2 content").unwrap();

    let config = IndexerConfig::new(app.id, db.clone(), true);
    let dir_hasher = DirHasher::new(config);
    let hash_result = dir_hasher
        .dir_hash(&Path::new(&test_dir).to_path_buf())
        .await
        .expect("failed to hash directory");
    let (hex_hash, changed_files) = hash_result.finalize().await;
    assert_eq!(hex_hash.len(), 64); // SHA-256 hash length in hex
    assert_eq!(
        hex_hash,
        "2ab14938127707cd534778654ef4d4400f9e26571acfe316074ead23155c734b"
    );
    assert_eq!(changed_files.len(), 4);

    // Now modify one file, delete another, and add a new file
    let inner_file3 = format!("{}/inner_file3.txt", sub_dir);
    fs::write(&outer_file, "Outer file 1 updated content").unwrap();
    fs::remove_file(&inner_file2).unwrap();
    fs::write(&inner_file3, "Inner file 3 content").unwrap();

    // Re-hash the directory
    let hash_result = dir_hasher
        .dir_hash(&Path::new(&test_dir).to_path_buf())
        .await
        .expect("failed to hash directory");
    let (hex_hash, changed_files) = hash_result.finalize().await;
    assert_eq!(hex_hash.len(), 64); // SHA-256 hash length in hex
    assert_eq!(
        hex_hash,
        "fad088f1c509fd120b2ab096178871743106368d81f992e59534f2534b04a36b"
    );
    assert_eq!(changed_files.len(), 4);
    verify_change(&outer_file, FileChangeType::Modified, &changed_files);
    verify_change(&sub_dir, FileChangeType::Modified, &changed_files);
    verify_change(&inner_file2, FileChangeType::Deleted, &changed_files);
    verify_change(&inner_file3, FileChangeType::Created, &changed_files);

    // // Verify data in the database
    verify_index(app.id, &test_dir, true, Some(&hex_hash), &db).await;
    verify_index(
        app.id,
        &outer_file,
        true,
        Some("711eb61f4cde35df5859281add666399cce1d2506dba6c01c0de58e315d93a57"),
        &db,
    )
    .await;
    verify_index(
        app.id,
        &sub_dir,
        true,
        Some("7610896fb921054ec3d0cdc3ca737fcd7b06caab41a093ba7a2a2720c1633b82"),
        &db,
    )
    .await;
    verify_index(
        app.id,
        &inner_file1,
        true,
        Some("eadae08b8cab3b95a3458a662af5591d314bd4e4525a7b5d6381aa56b5eda191"),
        &db,
    )
    .await;
    verify_index(app.id, &inner_file2, false, None, &db).await;
    verify_index(
        app.id,
        &inner_file3,
        true,
        Some("61fa2e094c8a3b784bf948e29cc7b593e21b9530eb1739744c2b5acdac7bfe50"),
        &db,
    )
    .await;
}

#[sqlx::test]
async fn dir_hasher_with_modified_nexted_dir(db_pool: SqlitePool) {
    let test_dir = initialize_test_dir("dir_hasher_with_modified_nexted_dir");
    let db = initialize_test_db(&db_pool).await;
    let app = initialize_test_app(&test_dir, &db).await;

    // Create a sub-directory with some files
    let outer_file = format!("{}/outer_file1.txt", test_dir);
    let sub_dir_level1 = format!("{}/level_1", test_dir);
    let sub_dir_level2 = format!("{}/level_2", sub_dir_level1);
    let inner_file = format!("{}/inner_file1.txt", sub_dir_level2);
    fs::write(&outer_file, "Outer file 1 content").unwrap();
    fs::create_dir_all(&sub_dir_level1).unwrap();
    fs::create_dir_all(&sub_dir_level2).unwrap();
    fs::write(&inner_file, "Inner file 1 content").unwrap();

    let config = IndexerConfig::new(app.id, db.clone(), true);
    let dir_hasher = DirHasher::new(config);
    let hash_result = dir_hasher
        .dir_hash(&Path::new(&test_dir).to_path_buf())
        .await
        .expect("failed to hash directory");
    let (hex_hash, changed_files) = hash_result.finalize().await;
    assert_eq!(hex_hash.len(), 64); // SHA-256 hash length in hex
    assert_eq!(
        hex_hash,
        "688540ba952dec4d91cd29a8ba08c23e7d6ea9a607d94a4d2ca535428c8db6b1"
    );
    assert_eq!(changed_files.len(), 4);
    verify_change(&outer_file, FileChangeType::Created, &changed_files);
    verify_change(&sub_dir_level1, FileChangeType::Created, &changed_files);
    verify_change(&sub_dir_level2, FileChangeType::Created, &changed_files);
    verify_change(&inner_file, FileChangeType::Created, &changed_files);

    // Now modify the inner file
    fs::write(&inner_file, "Inner file 1 updated content").unwrap();

    // Re-hash the directory
    let hash_result = dir_hasher
        .dir_hash(&Path::new(&test_dir).to_path_buf())
        .await
        .expect("failed to hash directory");
    let (hex_hash, changed_files) = hash_result.finalize().await;
    assert_eq!(hex_hash.len(), 64); // SHA-256 hash length in hex
    assert_eq!(
        hex_hash,
        "c8e4aaeec3d3561463ead6b985f8595ac4dbfaf1abc8a9b9379da99839df58dc"
    );
    assert_eq!(changed_files.len(), 3);
    verify_change(&sub_dir_level1, FileChangeType::Modified, &changed_files);
    verify_change(&sub_dir_level2, FileChangeType::Modified, &changed_files);
    verify_change(&inner_file, FileChangeType::Modified, &changed_files);

    // // Verify data in the database
    verify_index(app.id, &test_dir, true, Some(&hex_hash), &db).await;
    verify_index(
        app.id,
        &outer_file,
        true,
        Some("9058c9405a63ce79c2235326d65e409b12026f72e41b488af2af6b1020f51c85"),
        &db,
    )
    .await;
    verify_index(
        app.id,
        &sub_dir_level1,
        true,
        Some("258e0ada9cbcbefd3dcc750976e8c7a7733791dec26d2dd0e68302531d1d91d9"),
        &db,
    )
    .await;
    verify_index(
        app.id,
        &sub_dir_level2,
        true,
        Some("ac20d880d14574071debbc507fb19832b6366141da37be8aed9c6b45a36ea95a"),
        &db,
    )
    .await;
    verify_index(
        app.id,
        &inner_file,
        true,
        Some("fbb9f86652a9ad5dae1f7824aa13923727d76e9734ac766e8596d1e53180cfcf"),
        &db,
    )
    .await;
}
