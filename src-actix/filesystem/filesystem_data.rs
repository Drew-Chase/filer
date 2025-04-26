use anyhow::anyhow;
use serde::Serialize;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Serialize)]
pub struct FilesystemEntry {
    pub filename: String,
    pub path: String,
    pub size: u64,
    pub last_modified: Option<SystemTime>,
    pub created: Option<SystemTime>,
    pub is_dir: bool,
}

#[derive(Serialize)]
pub struct FilesystemData {
    pub parent: Option<String>,
    pub entries: Vec<FilesystemEntry>,
}

impl TryFrom<PathBuf> for FilesystemEntry {
    type Error = anyhow::Error;

    fn try_from(path: PathBuf) -> anyhow::Result<Self> {
        let metadata = path.metadata()?;
        let filename = path
            .file_name()
            .ok_or(anyhow!("Unable to get filename"))?
            .to_str()
            .ok_or(anyhow!("Unable to convert to str"))?
            .to_string();
        let path = path
            .to_str()
            .ok_or(anyhow!("Unable to convert to str"))?
            .to_string();
        let created = metadata.created().ok();
        let last_modified = metadata.modified().ok();
        Ok(FilesystemEntry {
            filename,
            path,
            created,
            last_modified,
            size: metadata.len(),
            is_dir: metadata.is_dir(),
        })
    }
}

impl TryFrom<PathBuf> for FilesystemData {
    type Error = anyhow::Error;

    fn try_from(path: PathBuf) -> anyhow::Result<Self> {
        if !path.exists() {
            return Err(anyhow::anyhow!("Path does not exist"));
        }
        let readdir = std::fs::read_dir(&path)?;
        let mut entries = Vec::new();
        for entry in readdir {
            let entry = entry?;
            let path = entry.path();
            if let Ok(entry) = path.try_into() {
                entries.push(entry);
            }
        }
        Ok(FilesystemData {
            parent: path.parent().map(|p| p.to_str().unwrap().to_string()),
            entries,
        })
    }
}
