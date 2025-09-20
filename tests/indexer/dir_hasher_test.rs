use std::{fs, path::Path};

use secret_online_patcher::indexer::{
    dir_hasher::DirHasher,
    file_change::{FileChange, FileChangeType},
    indexer_config::IndexerConfig,
};
use sqlx::SqlitePool;

use crate::common::test_util::{initialize_test_app, initialize_test_db, initialize_test_dir};

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
    fs::write(
        format!("{}/outer_file1.txt", test_dir),
        "Outer file 1 content",
    )
    .unwrap();
    let sub_dir = format!("{}/subdir", test_dir);
    fs::create_dir_all(&sub_dir).unwrap();
    fs::write(
        format!("{}/inner_file1.txt", sub_dir),
        "Inner file 1 content",
    )
    .unwrap();
    fs::write(
        format!("{}/inner_file2.txt", sub_dir),
        "Inner file 2 content",
    )
    .unwrap();

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
    verify_change(
        format!("{}/outer_file1.txt", test_dir).as_str(),
        FileChangeType::Created,
        &changed_files,
    );
    verify_change(
        format!("{}/subdir", test_dir).as_str(),
        FileChangeType::Created,
        &changed_files,
    );
    verify_change(
        format!("{}/inner_file1.txt", sub_dir).as_str(),
        FileChangeType::Created,
        &changed_files,
    );
    verify_change(
        format!("{}/inner_file2.txt", sub_dir).as_str(),
        FileChangeType::Created,
        &changed_files,
    );
}

#[sqlx::test]
async fn dir_hasher_with_modified_dir(db_pool: SqlitePool) {
    let test_dir = initialize_test_dir("dir_hasher_with_modified_dir");
    let db = initialize_test_db(&db_pool).await;
    let app = initialize_test_app(&test_dir, &db).await;

    // Create a sub-directory with some files
    fs::write(
        format!("{}/outer_file1.txt", test_dir),
        "Outer file 1 content",
    )
    .unwrap();
    let sub_dir = format!("{}/subdir", test_dir);
    fs::create_dir_all(&sub_dir).unwrap();
    fs::write(
        format!("{}/inner_file1.txt", sub_dir),
        "Inner file 1 content",
    )
    .unwrap();
    fs::write(
        format!("{}/inner_file2.txt", sub_dir),
        "Inner file 2 content",
    )
    .unwrap();

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
    fs::write(
        format!("{}/outer_file1.txt", test_dir),
        "Outer file 1 updated content",
    )
    .unwrap();
    fs::remove_file(format!("{}/inner_file2.txt", sub_dir)).unwrap();
    fs::write(
        format!("{}/inner_file3.txt", sub_dir),
        "Inner file 3 content",
    )
    .unwrap();

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
    verify_change(
        format!("{}/outer_file1.txt", test_dir).as_str(),
        FileChangeType::Modified,
        &changed_files,
    );
    verify_change(
        format!("{}/subdir", test_dir).as_str(),
        FileChangeType::Modified,
        &changed_files,
    );
    verify_change(
        format!("{}/inner_file2.txt", sub_dir).as_str(),
        FileChangeType::Deleted,
        &changed_files,
    );
    verify_change(
        format!("{}/inner_file3.txt", sub_dir).as_str(),
        FileChangeType::Created,
        &changed_files,
    );
}
