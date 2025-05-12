#[cfg(test)]
mod tests {
    use crate::filesystem::filesystem_data::FilesystemEntry;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_filesystem_entry_creation() {
        let now = SystemTime::now();
        let entry = FilesystemEntry {
            filename: "test_file.txt".to_string(),
            path: "/test/path".to_string(),
            is_dir: false,
            size: 1024,
            created: Some(now),
            last_modified: Some(now),
        };

        assert_eq!(entry.filename, "test_file.txt");
        assert_eq!(entry.path, "/test/path");
        assert_eq!(entry.is_dir, false);
        assert_eq!(entry.size, 1024);
        assert_eq!(entry.created, Some(now));
        assert_eq!(entry.last_modified, Some(now));
    }
}
