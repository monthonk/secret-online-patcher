use std::fmt::{Display, Formatter};

#[derive(PartialEq, Debug)]
pub enum FileChangeType {
    Created,
    Modified,
    Deleted,
}

pub struct FileChange {
    pub file_path: String,
    pub file_type: String,
    pub change_type: FileChangeType,
}

impl Display for FileChangeType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let change_str = match self {
            FileChangeType::Created => "Created",
            FileChangeType::Modified => "Modified",
            FileChangeType::Deleted => "Deleted",
        };
        write!(f, "{}", change_str)
    }
}
