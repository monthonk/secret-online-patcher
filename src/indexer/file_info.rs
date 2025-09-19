pub struct FileInfo {
    /// Absolute path to the file or directory
    pub _path: String,
    /// FILE or DIRECTORY
    pub file_type: String,
}

impl FileInfo {
    pub fn new(path: &str, file_type: &str) -> Self {
        FileInfo {
            _path: path.to_string(),
            file_type: file_type.to_string(),
        }
    }
}
