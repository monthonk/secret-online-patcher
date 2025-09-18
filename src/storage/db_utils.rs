/// A utility module for database operations.
use std::path::Path;

use crate::storage::{file_index::FileIndex, patcher_db::PatcherDatabase};

pub async fn last_index(app_id: i64, file_path: &Path, db: &PatcherDatabase) -> Option<FileIndex> {
    let file_path = file_path.display().to_string();
    db.get_file_index(app_id, &file_path).await.unwrap_or(None)
}

/// List all direct indexed files under a given directory for an application.
pub async fn list_indexed_files(
    app_id: i64,
    parent_dir: &Path,
    db: &PatcherDatabase,
) -> Result<Vec<FileIndex>, sqlx::Error> {
    let dir_path = parent_dir.display().to_string();
    let files = db.get_files_in_directory(app_id, &dir_path).await?;
    let direct_children = get_direct_children(parent_dir, &files);
    Ok(direct_children)
}

fn get_direct_children(parent: &Path, all_files: &[FileIndex]) -> Vec<FileIndex> {
    let mut children = Vec::new();
    for file in all_files {
        let file_path = Path::new(&file.file_path);
        if let Some(parent_path) = file_path.parent() {
            if parent_path == parent {
                children.push(file.clone());
            }
        }
    }
    children
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::storage::file_index::FileIndex;

    fn verify_file_index(files: &[FileIndex], expected_path: &str, expected_type: &str) {
        let file = files
            .iter()
            .find(|f| f.file_path == expected_path)
            .expect("File not found");
        assert_eq!(file.file_path, expected_path);
        assert_eq!(file.file_type, expected_type);
    }

    #[test]
    fn get_direct_children_test() {
        let parrent = Path::new("/var/test/");
        let files = vec![
            FileIndex::mock("/var/test/file1.txt", "FILE"),
            FileIndex::mock("/var/test/file2.txt", "FILE"),
            FileIndex::mock("/var/test/subdir", "DIRECTORY"),
            FileIndex::mock("/var/test/subdir/file3.txt", "FILE"),
            FileIndex::mock("/var/test/subdir2", "DIRECTORY"),
            FileIndex::mock("/var/test/subdir2/file4.txt", "FILE"),
            FileIndex::mock("/var/test/subdir2/nested", "DIRECTORY"),
            FileIndex::mock("/var/test/subdir2/nested/file5.txt", "FILE"),
        ];

        let children = super::get_direct_children(parrent, &files);
        assert_eq!(children.len(), 4);
        verify_file_index(&children, "/var/test/file1.txt", "FILE");
        verify_file_index(&children, "/var/test/file2.txt", "FILE");
        verify_file_index(&children, "/var/test/subdir", "DIRECTORY");
        verify_file_index(&children, "/var/test/subdir2", "DIRECTORY");
    }
}
