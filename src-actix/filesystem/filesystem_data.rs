use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct FilesystemEntry {
    pub filename: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
}

#[derive(Serialize)]
pub struct FilesystemData {
    pub parent: Option<String>,
    pub entries: Vec<FilesystemEntry>,
}

impl From<PathBuf> for FilesystemEntry {
    fn from(path: PathBuf) -> Self {
        let metadata = path.metadata().unwrap();
        FilesystemEntry {
            filename: path.file_name().unwrap().to_str().unwrap().to_string(),
            path: path.to_str().unwrap().to_string(),
            size: metadata.len(),
            is_dir: metadata.is_dir(),
        }
    }
}

impl From<PathBuf> for FilesystemData {
    fn from(path: PathBuf) -> Self {
        let readdir = std::fs::read_dir(&path).unwrap();
        let mut entries = Vec::new();
        for entry in readdir {
            let entry = entry.unwrap();
            let path = entry.path();
            entries.push(path.into());
        }
        FilesystemData {
            parent: path.parent().map(|p| p.to_str().unwrap().to_string()),
            entries,
        }
    }
}
