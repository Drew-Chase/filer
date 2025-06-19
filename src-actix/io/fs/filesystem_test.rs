#[cfg(test)]
mod tests {
    use crate::io::fs::filesystem_data::FilesystemEntry;
    use std::time::SystemTime;
    use std::io::{Cursor, Read, Write};
    use std::path::{Path, PathBuf};
    use std::fs::File;
    use tempfile::tempdir;
    use zip::{ZipWriter, CompressionMethod};
    use zip::write::SimpleFileOptions;
    use walkdir::WalkDir;

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

    #[test]
    fn test_zip_archive_creation() {
        // Create a temporary directory for our test files
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        // Create a test file
        let test_file_path = temp_path.join("test_file.txt");
        let mut test_file = File::create(&test_file_path).expect("Failed to create test file");
        test_file.write_all(b"This is test content").expect("Failed to write to test file");

        // Create a subdirectory
        let subdir_path = temp_path.join("subdir");
        std::fs::create_dir(&subdir_path).expect("Failed to create subdirectory");

        // Create a test file in the subdirectory
        let subdir_file_path = subdir_path.join("subdir_file.txt");
        let mut subdir_file = File::create(&subdir_file_path).expect("Failed to create subdir file");
        subdir_file.write_all(b"This is content in a subdirectory").expect("Failed to write to subdir file");

        // Create an in-memory zip archive
        let in_memory_buffer = Cursor::new(Vec::new());
        let mut zip = ZipWriter::new(in_memory_buffer);
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated);

        // Add the test file to the zip
        let relative_path = "test_file.txt";
        zip.start_file(relative_path, options).expect("Failed to start file in zip");
        let mut file = File::open(&test_file_path).expect("Failed to open test file");
        let mut buffer = vec![0; 1024];
        loop {
            let bytes_read = file.read(&mut buffer).expect("Failed to read file");
            if bytes_read == 0 { break; }
            zip.write_all(&buffer[..bytes_read]).expect("Failed to write to zip");
        }

        // Add the subdirectory to the zip
        let subdir_rel_path = "subdir/";
        zip.add_directory(subdir_rel_path, options).expect("Failed to add directory to zip");

        // Add the subdirectory file to the zip
        let subdir_file_rel_path = "subdir/subdir_file.txt";
        zip.start_file(subdir_file_rel_path, options).expect("Failed to start subdir file in zip");
        let mut subdir_file = File::open(&subdir_file_path).expect("Failed to open subdir file");
        loop {
            let bytes_read = subdir_file.read(&mut buffer).expect("Failed to read subdir file");
            if bytes_read == 0 { break; }
            zip.write_all(&buffer[..bytes_read]).expect("Failed to write subdir file to zip");
        }

        // Finish the zip
        let cursor = zip.finish().expect("Failed to finish zip");
        let zip_data = cursor.into_inner();

        // Verify the zip is not empty
        assert!(zip_data.len() > 0, "Zip archive is empty");
        println!("Zip archive size: {} bytes", zip_data.len());

        // Verify we can read the zip
        let reader = Cursor::new(zip_data);
        let mut zip_archive = zip::ZipArchive::new(reader).expect("Failed to read zip archive");

        // Check the number of files in the zip
        assert_eq!(zip_archive.len(), 3, "Zip should contain 3 entries (1 file, 1 directory, 1 file in directory)");

        // Check the contents of the files
        {
            let mut file = zip_archive.by_name("test_file.txt").expect("Failed to find test_file.txt in zip");
            let mut contents = String::new();
            file.read_to_string(&mut contents).expect("Failed to read test_file.txt from zip");
            assert_eq!(contents, "This is test content", "File content doesn't match");
        }

        {
            let mut subdir_file = zip_archive.by_name("subdir/subdir_file.txt").expect("Failed to find subdir_file.txt in zip");
            let mut subdir_contents = String::new();
            subdir_file.read_to_string(&mut subdir_contents).expect("Failed to read subdir_file.txt from zip");
            assert_eq!(subdir_contents, "This is content in a subdirectory", "Subdir file content doesn't match");
        }
    }

    #[test]
    fn test_multi_path_zip_creation() {
        // Create a temporary directory structure that mimics the real-world scenario
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let base_dir = temp_dir.path();

        // Create a directory structure like:
        // base_dir/
        //   ├── test/
        //   │   ├── file1.txt
        //   │   ├── subdir/
        //   │   │   └── file2.txt
        //   └── test.csv

        // Create the test directory
        let test_dir = base_dir.join("test");
        std::fs::create_dir(&test_dir).expect("Failed to create test directory");

        // Create a file in the test directory
        let file1_path = test_dir.join("file1.txt");
        let mut file1 = File::create(&file1_path).expect("Failed to create file1.txt");
        file1.write_all(b"Content of file1").expect("Failed to write to file1.txt");

        // Create a subdirectory in the test directory
        let subdir_path = test_dir.join("subdir");
        std::fs::create_dir(&subdir_path).expect("Failed to create subdirectory");

        // Create a file in the subdirectory
        let file2_path = subdir_path.join("file2.txt");
        let mut file2 = File::create(&file2_path).expect("Failed to create file2.txt");
        file2.write_all(b"Content of file2").expect("Failed to write to file2.txt");

        // Create a file in the base directory
        let csv_path = base_dir.join("test.csv");
        let mut csv_file = File::create(&csv_path).expect("Failed to create test.csv");
        csv_file.write_all(b"a,b,c\n1,2,3").expect("Failed to write to test.csv");

        // Now simulate the multi-path zip creation process
        let paths = vec![
            test_dir.to_string_lossy().to_string(),
            csv_path.to_string_lossy().to_string(),
        ];

        println!("Paths to process: {:?}", paths);

        // Find the common base path
        let common_base = {
            let path_bufs: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();

            if path_bufs.is_empty() {
                panic!("No valid paths provided");
            }

            // Get the parent directory of the first path
            let first_path = &path_bufs[0];
            let mut common_parent = if first_path.is_file() {
                first_path.parent().unwrap_or(Path::new("")).to_path_buf()
            } else {
                first_path.clone()
            };

            // Find the common parent directory
            for path in &path_bufs[1..] {
                let path_parent = if path.is_file() {
                    path.parent().unwrap_or(Path::new("")).to_path_buf()
                } else {
                    path.clone()
                };

                // Find the common parent by comparing components
                let common_components: Vec<_> = common_parent
                    .components()
                    .zip(path_parent.components())
                    .take_while(|(a, b)| a == b)
                    .map(|(a, _)| a)
                    .collect();

                if common_components.is_empty() {
                    // No common parent, use the drive root
                    if let Some(drive) = common_parent.components().next() {
                        common_parent = PathBuf::from(drive.as_os_str());
                    } else {
                        common_parent = PathBuf::new();
                    }
                    break;
                } else {
                    // Rebuild the common parent from components
                    common_parent = common_components.iter().fold(PathBuf::new(), |mut path, component| {
                        path.push(component.as_os_str());
                        path
                    });
                }
            }

            // Convert to string and ensure it ends with a separator
            let mut common_prefix = common_parent.to_string_lossy().to_string();
            if !common_prefix.is_empty() && !common_prefix.ends_with('\\') {
                common_prefix.push('\\');
            }

            common_prefix
        };

        println!("Common base path: {}", common_base);

        // Create an in-memory zip archive
        let in_memory_buffer = Cursor::new(Vec::new());
        let mut zip = ZipWriter::new(in_memory_buffer);
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated);

        // Use the common base path we found earlier
        let base_path = PathBuf::from(&common_base);
        let buffer_size = 64 * 1024; // 64KB buffer
        let mut read_buffer = vec![0; buffer_size];

        // Keep track of directories we've already added
        let mut added_directories = std::collections::HashSet::new();

        // Process each path
        let path_bufs: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
        println!("Processing {} paths", path_bufs.len());

        for (i, path) in path_bufs.iter().enumerate() {
            println!("Processing path {}/{}: {}", i + 1, path_bufs.len(), path.display());

            if path.is_dir() {
                println!("Path is a directory: {}", path.display());
                // Process directory
                let dir = WalkDir::new(path);
                let mut file_count = 0;

                for entry_result in dir {
                    let dir_entry = entry_result.expect("Failed to read directory");
                    let entry_path = dir_entry.path();
                    file_count += 1;
                    println!("Found entry {}: {}", file_count, entry_path.display());

                    // Create a relative path
                    let entry_path_str = entry_path.to_string_lossy();
                    let base_path_str = base_path.to_string_lossy();
                    println!("Entry path: {}", entry_path_str);
                    println!("Base path: {}", base_path_str);

                    if entry_path_str.starts_with(&*base_path_str) {
                        let relative_path = &entry_path_str[base_path_str.len()..];
                        println!("Relative path before processing: {}", relative_path);
                        // Replace backslashes with forward slashes and ensure no leading slash
                        let relative_path = relative_path.replace('\\', "/");
                        let relative_path = relative_path.trim_start_matches('/');
                        println!("Relative path after processing: {}", relative_path);

                        if entry_path.is_dir() {
                            // Add directory entry to the zip
                            let dir_path = if relative_path.is_empty() {
                                String::from("")
                            } else {
                                format!("{}/", relative_path)
                            };

                            // Check if this directory has already been added
                            if !added_directories.contains(&dir_path) {
                                println!("Adding directory to zip: {}", dir_path);
                                zip.add_directory(&dir_path, options).expect("Failed to create directory entry");
                                added_directories.insert(dir_path);
                            } else {
                                println!("Directory already exists in zip: {}", dir_path);
                            }
                        } else if entry_path.is_file() {
                            // Create parent directory entries if needed
                            if let Some(parent) = Path::new(&relative_path).parent() {
                                if !parent.as_os_str().is_empty() {
                                    let parent_path = parent.to_string_lossy().replace('\\', "/");
                                    if !parent_path.is_empty() {
                                        let dir_path = format!("{}/", parent_path);
                                        // Check if this directory has already been added
                                        if !added_directories.contains(&dir_path) {
                                            println!("Adding parent directory to zip: {}", dir_path);
                                            zip.add_directory(&dir_path, options).expect("Failed to create directory entry");
                                            added_directories.insert(dir_path);
                                        } else {
                                            println!("Directory already exists in zip: {}", dir_path);
                                        }
                                    }
                                }
                            }

                            // Start a new file in the zip
                            println!("Adding file to zip: {}", relative_path);
                            zip.start_file(relative_path, options).expect("Failed to create zip entry");

                            // Open the file
                            let mut file = File::open(entry_path).expect("Failed to open file");

                            // Read and write the file in chunks
                            let mut total_bytes_written = 0;
                            loop {
                                let bytes_read = file.read(&mut read_buffer).expect("Failed to read file");

                                if bytes_read == 0 { break; }

                                zip.write_all(&read_buffer[..bytes_read]).expect("Failed to write to zip");

                                total_bytes_written += bytes_read;
                            }
                            println!("Wrote {} bytes to zip for file: {}", total_bytes_written, relative_path);
                        }
                    }
                }
            } else if path.is_file() {
                println!("Path is a file: {}", path.display());
                // Process single file
                // Create a relative path
                let path_str = path.to_string_lossy();
                let base_path_str = base_path.to_string_lossy();
                println!("Base path: {}", base_path_str);

                if path_str.starts_with(&*base_path_str) {
                    let relative_path = &path_str[base_path_str.len()..];
                    println!("Relative path before processing: {}", relative_path);
                    // Replace backslashes with forward slashes and ensure no leading slash
                    let relative_path = relative_path.replace('\\', "/");
                    let relative_path = relative_path.trim_start_matches('/');
                    println!("Relative path after processing: {}", relative_path);

                    // Create parent directory entries if needed
                    if let Some(parent) = Path::new(&relative_path).parent() {
                        if !parent.as_os_str().is_empty() {
                            let parent_path = parent.to_string_lossy().replace('\\', "/");
                            if !parent_path.is_empty() {
                                let dir_path = format!("{}/", parent_path);
                                // Check if this directory has already been added
                                if !added_directories.contains(&dir_path) {
                                    println!("Adding parent directory to zip: {}", dir_path);
                                    zip.add_directory(&dir_path, options).expect("Failed to create directory entry");
                                    added_directories.insert(dir_path);
                                } else {
                                    println!("Directory already exists in zip: {}", dir_path);
                                }
                            }
                        }
                    }

                    // Start a new file in the zip
                    println!("Adding file to zip: {}", relative_path);
                    zip.start_file(relative_path, options).expect("Failed to create zip entry");

                    // Open the file
                    let mut file = File::open(path).expect("Failed to open file");

                    // Read and write the file in chunks
                    let mut total_bytes_written = 0;
                    loop {
                        let bytes_read = file.read(&mut read_buffer).expect("Failed to read file");

                        if bytes_read == 0 { break; }

                        zip.write_all(&read_buffer[..bytes_read]).expect("Failed to write to zip");

                        total_bytes_written += bytes_read;
                    }
                    println!("Wrote {} bytes to zip for file: {}", total_bytes_written, relative_path);
                }
            }
        }

        // Finish the zip file (writes the central directory)
        let cursor = zip.finish().expect("Failed to finish zip");

        // Get the final zip data
        let zip_data = cursor.into_inner();

        // Log the size of the zip data
        println!("Multi-path zip data size: {} bytes", zip_data.len());

        // Verify the zip is not empty
        assert!(zip_data.len() > 0, "Zip archive is empty");

        // Verify we can read the zip
        let reader = Cursor::new(zip_data);
        let mut zip_archive = zip::ZipArchive::new(reader).expect("Failed to read zip archive");

        // Print all files in the zip
        println!("Files in the zip archive:");
        for i in 0..zip_archive.len() {
            let file = zip_archive.by_index(i).expect("Failed to get file by index");
            println!("  {}: {} bytes", file.name(), file.size());
        }

        // Check the number of files in the zip
        assert!(zip_archive.len() >= 4, "Zip should contain at least 4 entries (test dir, file1.txt, subdir, file2.txt, test.csv)");

        // Check that we can read the files
        {
            let mut file = zip_archive.by_name("test.csv").expect("Failed to find test.csv in zip");
            let mut contents = String::new();
            file.read_to_string(&mut contents).expect("Failed to read test.csv from zip");
            assert_eq!(contents, "a,b,c\n1,2,3", "CSV file content doesn't match");
        }

        {
            let mut file = zip_archive.by_name("test/file1.txt").expect("Failed to find test/file1.txt in zip");
            let mut contents = String::new();
            file.read_to_string(&mut contents).expect("Failed to read test/file1.txt from zip");
            assert_eq!(contents, "Content of file1", "File1 content doesn't match");
        }

        {
            let mut file = zip_archive.by_name("test/subdir/file2.txt").expect("Failed to find test/subdir/file2.txt in zip");
            let mut contents = String::new();
            file.read_to_string(&mut contents).expect("Failed to read test/subdir/file2.txt from zip");
            assert_eq!(contents, "Content of file2", "File2 content doesn't match");
        }
    }
}

#[cfg(test)]
mod endpoint_tests {
    use actix_web::{test, web, App, http::header};
    use crate::io::fs::filesystem_endpoint;
    use crate::configuration::configuration_data::Configuration;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::{Read, Write};
    use std::path::PathBuf;

    // Test for get_filesystem_entries endpoint
    #[actix_web::test]
    async fn test_get_filesystem_entries() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        // Create a test file
        let test_file_path = temp_path.join("test_file.txt");
        let mut test_file = File::create(&test_file_path).expect("Failed to create test file");
        test_file.write_all(b"This is test content").expect("Failed to write to test file");

        // Create a test app
        let app = test::init_service(
            App::new()
                .service(
                    web::scope("/api")
                        .service(
                            web::scope("/fs")
                                .service(filesystem_endpoint::get_filesystem_entries)
                        )
                )
        ).await;

        // Create a test request
        let req = test::TestRequest::get()
            .uri("/api/fs/")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .insert_header(("X-Filesystem-Path", temp_path.to_string_lossy().to_string()))
            .to_request();

        // Send the request and get the response
        let resp = test::call_and_read_body(&app, req).await;

        // Parse the response body
        let json: serde_json::Value = serde_json::from_slice(&resp).expect("Failed to parse response JSON");

        // Check that the response contains the expected data
        assert!(json.get("entries").is_some());
        let entries = json.get("entries").unwrap().as_array().unwrap();

        // Check that our test file is in the entries
        let found_test_file = entries.iter().any(|entry| {
            entry.get("filename").unwrap().as_str().unwrap() == "test_file.txt"
        });

        assert!(found_test_file, "Test file not found in response");
    }

    // Test for download endpoint (single file)
    #[actix_web::test]
    async fn test_download_file() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        // Create a test file
        let test_file_path = temp_path.join("test_file.txt");
        let test_content = b"This is test content";
        let mut test_file = File::create(&test_file_path).expect("Failed to create test file");
        test_file.write_all(test_content).expect("Failed to write to test file");

        // Create a test app
        let app = test::init_service(
            App::new()
                .service(
                    web::scope("/api")
                        .service(
                            web::scope("/fs")
                                .service(filesystem_endpoint::download)
                        )
                )
        ).await;

        // Create a test request
        let req = test::TestRequest::get()
            .uri("/api/fs/download")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .insert_header(("X-Filesystem-Path", test_file_path.to_string_lossy().to_string()))
            .to_request();

        // Send the request and get the response
        let resp = test::call_service(&app, req).await;

        // Check that the response is successful
        assert!(resp.status().is_success());

        // Check the content type
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap().to_str().unwrap(),
            "application/octet-stream"
        );

        // Check that the Content-Disposition header is set correctly
        let content_disposition = resp.headers().get(header::CONTENT_DISPOSITION).unwrap().to_str().unwrap();
        assert!(content_disposition.contains("attachment"));
        assert!(content_disposition.contains("filename=\"test_file.txt\""));

        // Read the response body
        let body = test::read_body(resp).await;

        // Check that the body contains the expected content
        assert_eq!(body.as_ref(), test_content);
    }

    // Test for download endpoint (directory)
    #[actix_web::test]
    async fn test_download_directory() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        // Create a test file in the directory
        let test_file_path = temp_path.join("test_file.txt");
        let test_content = b"This is test content";
        let mut test_file = File::create(&test_file_path).expect("Failed to create test file");
        test_file.write_all(test_content).expect("Failed to write to test file");

        // Create a subdirectory
        let subdir_path = temp_path.join("subdir");
        std::fs::create_dir(&subdir_path).expect("Failed to create subdirectory");

        // Create a test file in the subdirectory
        let subdir_file_path = subdir_path.join("subdir_file.txt");
        let subdir_content = b"This is content in a subdirectory";
        let mut subdir_file = File::create(&subdir_file_path).expect("Failed to create subdir file");
        subdir_file.write_all(subdir_content).expect("Failed to write to subdir file");

        // Create a test app
        let app = test::init_service(
            App::new()
                .service(
                    web::scope("/api")
                        .service(
                            web::scope("/fs")
                                .service(filesystem_endpoint::download)
                        )
                )
        ).await;

        // Create a test request
        let req = test::TestRequest::get()
            .uri("/api/fs/download")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .insert_header(("X-Filesystem-Path", temp_path.to_string_lossy().to_string()))
            .to_request();

        // Send the request and get the response
        let resp = test::call_service(&app, req).await;

        // Check that the response is successful
        assert!(resp.status().is_success());

        // Check the content type
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap().to_str().unwrap(),
            "application/octet-stream"
        );

        // Check that the Content-Disposition header is set correctly
        let content_disposition = resp.headers().get(header::CONTENT_DISPOSITION).unwrap().to_str().unwrap();
        assert!(content_disposition.contains("attachment"));

        // Read the response body (zip file)
        let body = test::read_body(resp).await;

        // Verify the zip is not empty
        assert!(!body.is_empty(), "Zip archive is empty");

        // Verify we can read the zip
        let reader = std::io::Cursor::new(body);
        let mut zip_archive = zip::ZipArchive::new(reader).expect("Failed to read zip archive");

        // Check that the zip contains our files
        assert!(zip_archive.len() >= 3, "Zip should contain at least 3 entries (root dir, file, subdir)");

        // Check the contents of the files in the zip
        let file_names: Vec<String> = (0..zip_archive.len())
            .map(|i| zip_archive.by_index(i).unwrap().name().to_string())
            .collect();

        // Check for the test file
        let test_file_found = file_names.iter().any(|name| name.ends_with("test_file.txt"));
        assert!(test_file_found, "test_file.txt not found in zip");

        // Check for the subdirectory file
        let subdir_file_found = file_names.iter().any(|name| name.contains("subdir") && name.ends_with("subdir_file.txt"));
        assert!(subdir_file_found, "subdir/subdir_file.txt not found in zip");
    }

    // Test for download endpoint (multiple files)
    #[actix_web::test]
    async fn test_download_multiple_files() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        // Create first test file
        let test_file1_path = temp_path.join("test_file1.txt");
        let test_content1 = b"This is test content 1";
        let mut test_file1 = File::create(&test_file1_path).expect("Failed to create test file 1");
        test_file1.write_all(test_content1).expect("Failed to write to test file 1");

        // Create second test file
        let test_file2_path = temp_path.join("test_file2.txt");
        let test_content2 = b"This is test content 2";
        let mut test_file2 = File::create(&test_file2_path).expect("Failed to create test file 2");
        test_file2.write_all(test_content2).expect("Failed to write to test file 2");

        // Create a test app
        let app = test::init_service(
            App::new()
                .service(
                    web::scope("/api")
                        .service(
                            web::scope("/fs")
                                .service(filesystem_endpoint::download)
                        )
                )
        ).await;

        // Create paths array for X-Multiple-Paths header
        let paths = vec![
            test_file1_path.to_string_lossy().to_string(),
            test_file2_path.to_string_lossy().to_string(),
        ];
        let paths_json = serde_json::to_string(&paths).expect("Failed to serialize paths to JSON");

        // Create a test request
        let req = test::TestRequest::get()
            .uri("/api/fs/download")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .insert_header(("X-Multiple-Paths", paths_json))
            .to_request();

        // Send the request and get the response
        let resp = test::call_service(&app, req).await;

        // Check that the response is successful
        assert!(resp.status().is_success());

        // Check the content type
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap().to_str().unwrap(),
            "application/octet-stream"
        );

        // Check that the Content-Disposition header is set correctly
        let content_disposition = resp.headers().get(header::CONTENT_DISPOSITION).unwrap().to_str().unwrap();
        assert!(content_disposition.contains("attachment"));
        assert!(content_disposition.contains(".zip"), "Filename should have .zip extension");

        // Read the response body (zip file)
        let body = test::read_body(resp).await;

        // Verify the zip is not empty
        assert!(!body.is_empty(), "Zip archive is empty");

        // Verify we can read the zip
        let reader = std::io::Cursor::new(body);
        let mut zip_archive = zip::ZipArchive::new(reader).expect("Failed to read zip archive");

        // Check that the zip contains our files
        assert_eq!(zip_archive.len(), 2, "Zip should contain 2 entries (the two files)");

        // Check the contents of the files in the zip
        let file_names: Vec<String> = (0..zip_archive.len())
            .map(|i| zip_archive.by_index(i).unwrap().name().to_string())
            .collect();

        // Check for the test files
        let test_file1_found = file_names.iter().any(|name| name.ends_with("test_file1.txt"));
        let test_file2_found = file_names.iter().any(|name| name.ends_with("test_file2.txt"));
        assert!(test_file1_found, "test_file1.txt not found in zip");
        assert!(test_file2_found, "test_file2.txt not found in zip");

        // Verify the content of the files
        {
            let mut file = zip_archive.by_name(file_names.iter().find(|name| name.ends_with("test_file1.txt")).unwrap())
                .expect("Failed to open test_file1.txt in zip");
            let mut content = Vec::new();
            file.read_to_end(&mut content).expect("Failed to read test_file1.txt from zip");
            assert_eq!(content, test_content1, "Content of test_file1.txt doesn't match");
        }

        {
            let mut file = zip_archive.by_name(file_names.iter().find(|name| name.ends_with("test_file2.txt")).unwrap())
                .expect("Failed to open test_file2.txt in zip");
            let mut content = Vec::new();
            file.read_to_end(&mut content).expect("Failed to read test_file2.txt from zip");
            assert_eq!(content, test_content2, "Content of test_file2.txt doesn't match");
        }
    }

    // Test for search endpoint
    #[actix_web::test]
    async fn test_search() {
        // Create a test app
        let app = test::init_service(
            App::new()
                .service(
                    web::scope("/api")
                        .service(
                            web::scope("/fs")
                                .service(filesystem_endpoint::search)
                        )
                )
        ).await;

        // Test with a valid query
        let req = test::TestRequest::get()
            .uri("/api/fs/search?q=test")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Test with an invalid query (missing q parameter)
        let req = test::TestRequest::get()
            .uri("/api/fs/search")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 400); // Bad Request
    }

    // Test for delete endpoint
    #[actix_web::test]
    async fn test_delete_filesystem_entry() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        // Create a test file
        let test_file_path = temp_path.join("test_file.txt");
        let mut test_file = File::create(&test_file_path).expect("Failed to create test file");
        test_file.write_all(b"This is test content").expect("Failed to write to test file");

        // Create a test app
        let app = test::init_service(
            App::new()
                .service(
                    web::scope("/api")
                        .service(
                            web::scope("/fs")
                                .service(filesystem_endpoint::delete_filesystem_entry)
                        )
                )
        ).await;

        // Create a test request
        let req = test::TestRequest::delete()
            .uri("/api/fs/")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .insert_header(("X-Filesystem-Path", test_file_path.to_string_lossy().to_string()))
            .to_request();

        // Send the request and get the response
        let resp = test::call_service(&app, req).await;

        // Check that the response is successful
        assert!(resp.status().is_success());

        // Check that the file was deleted
        assert!(!test_file_path.exists());
    }

    // Test for copy endpoint
    #[actix_web::test]
    async fn test_copy_filesystem_entry() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        // Create a test file
        let test_file_path = temp_path.join("test_file.txt");
        let test_content = b"This is test content";
        let mut test_file = File::create(&test_file_path).expect("Failed to create test file");
        test_file.write_all(test_content).expect("Failed to write to test file");

        // Define the destination path
        let dest_file_path = temp_path.join("test_file_copy.txt");

        // Create a test app
        let app = test::init_service(
            App::new()
                .service(
                    web::scope("/api")
                        .service(
                            web::scope("/fs")
                                .service(filesystem_endpoint::copy_filesystem_entry)
                        )
                )
        ).await;

        // Create a test request
        let req = test::TestRequest::post()
            .uri("/api/fs/copy")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .insert_header(("X-Filesystem-Path", test_file_path.to_string_lossy().to_string()))
            .insert_header(("X-NewFilesystem-Path", dest_file_path.to_string_lossy().to_string()))
            .to_request();

        // Send the request and get the response
        let resp = test::call_service(&app, req).await;

        // Check that the response is successful
        assert!(resp.status().is_success());

        // Check that the destination file exists
        assert!(dest_file_path.exists());

        // Read the destination file and check its content
        let dest_content = std::fs::read(&dest_file_path).expect("Failed to read destination file");
        assert_eq!(dest_content, test_content);
    }

    // Test for upload_progress endpoint
    #[actix_web::test]
    async fn test_upload_progress() {
        // Create a test app
        let app = test::init_service(
            App::new()
                .service(
                    web::scope("/api")
                        .service(
                            web::scope("/fs")
                                .service(filesystem_endpoint::upload_progress)
                        )
                )
        ).await;

        // Create a test request with a test upload ID
        let upload_id = "test-upload-id";
        let req = test::TestRequest::get()
            .uri(&format!("/api/fs/upload/progress/{}", upload_id))
            .to_request();

        // Send the request and get the response
        let resp = test::call_service(&app, req).await;

        // Check that the response is successful
        assert!(resp.status().is_success());

        // Check that the content type is text/event-stream for SSE
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap().to_str().unwrap(),
            "text/event-stream"
        );
    }

    // Test for upload endpoint
    #[actix_web::test]
    async fn test_upload() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        // Define the upload file path
        let upload_path = temp_path.join("uploaded_file.txt");

        // Create a test app
        let app = test::init_service(
            App::new()
                .service(
                    web::scope("/api")
                        .service(
                            web::scope("/fs")
                                .service(filesystem_endpoint::upload)
                        )
                )
        ).await;

        // Create test content
        let test_content = b"This is test content for upload";

        // Create a test request
        let req = test::TestRequest::post()
            .uri("/api/fs/upload")
            .insert_header((header::CONTENT_TYPE, "application/octet-stream"))
            .insert_header(("X-Filesystem-Path", upload_path.to_string_lossy().to_string()))
            .insert_header(("X-Upload-ID", "test-upload-id"))
            .set_payload(test_content.to_vec())
            .to_request();

        // Send the request and get the response
        let resp = test::call_service(&app, req).await;

        // Check that the response is successful
        assert!(resp.status().is_success());

        // Check that the file was created
        assert!(upload_path.exists());

        // Read the uploaded file and check its content
        let uploaded_content = std::fs::read(&upload_path).expect("Failed to read uploaded file");
        assert_eq!(uploaded_content, test_content);
    }

    // Test for root_path configuration
    #[actix_web::test]
    async fn test_root_path_configuration() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        // Create a test file in the temporary directory
        let test_file_path = temp_path.join("test_file.txt");
        let test_content = b"This is test content for root_path test";
        let mut test_file = File::create(&test_file_path).expect("Failed to create test file");
        test_file.write_all(test_content).expect("Failed to write to test file");

        // Set the root_path configuration to the temporary directory
        let mut config = Configuration::get().clone();
        let original_root_path = config.root_path.clone();
        config.root_path = temp_path.to_string_lossy().to_string();

        // Create a test app with a custom configuration
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(config.clone()))
                .service(
                    web::scope("/api")
                        .service(
                            web::scope("/fs")
                                .service(filesystem_endpoint::get_filesystem_entries)
                        )
                )
        ).await;

        // Create a test request with path "/"
        let req = test::TestRequest::get()
            .uri("/api/fs/")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .insert_header(("X-Filesystem-Path", "/"))
            .to_request();

        // Send the request and get the response
        let resp = test::call_and_read_body(&app, req).await;

        // Parse the response body
        let json: serde_json::Value = serde_json::from_slice(&resp).expect("Failed to parse response JSON");

        // Check that the response contains the expected data
        assert!(json.get("entries").is_some());
        let entries = json.get("entries").unwrap().as_array().unwrap();

        // Check that our test file is in the entries
        let found_test_file = entries.iter().any(|entry| {
            entry.get("filename").unwrap().as_str().unwrap() == "test_file.txt"
        });

        assert!(found_test_file, "Test file not found in response");

        // Reset the root_path configuration to its original value
        let mut config = Configuration::get().clone();
        config.root_path = original_root_path;
    }

    // Test for move endpoint
    #[actix_web::test]
    async fn test_move_filesystem_entry() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        // Create a test file
        let test_file_path = temp_path.join("test_file.txt");
        let test_content = b"This is test content";
        let mut test_file = File::create(&test_file_path).expect("Failed to create test file");
        test_file.write_all(test_content).expect("Failed to write to test file");

        // Define the destination path
        let dest_file_path = temp_path.join("test_file_moved.txt");

        // Create a test app
        let app = test::init_service(
            App::new()
                .service(
                    web::scope("/api")
                        .service(
                            web::scope("/fs")
                                .service(filesystem_endpoint::move_filesystem_entry)
                        )
                )
        ).await;

        // Create a test request
        let req = test::TestRequest::post()
            .uri("/api/fs/move")
            .insert_header((header::CONTENT_TYPE, "application/json"))
            .insert_header(("X-Filesystem-Path", test_file_path.to_string_lossy().to_string()))
            .insert_header(("X-NewFilesystem-Path", dest_file_path.to_string_lossy().to_string()))
            .to_request();

        // Send the request and get the response
        let resp = test::call_service(&app, req).await;

        // Check that the response is successful
        assert!(resp.status().is_success());

        // Check that the source file no longer exists
        assert!(!test_file_path.exists());

        // Check that the destination file exists
        assert!(dest_file_path.exists());

        // Read the destination file and check its content
        let dest_content = std::fs::read(&dest_file_path).expect("Failed to read destination file");
        assert_eq!(dest_content, test_content);
    }
}
