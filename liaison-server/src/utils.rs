// Utility functions for file operations
// Corresponds to utils.cpp in the C++ version

use anyhow::Result;
use std::path::Path;

/// Create directories recursively
pub fn create_directories<P: AsRef<Path>>(path: P) -> Result<()> {
    std::fs::create_dir_all(path)?;
    Ok(())
}

/// Create a temporary directory for FMU extraction
pub fn create_temp_directory() -> Result<String> {
    let temp_dir = tempfile::tempdir()?;
    let path = temp_dir.path().to_string_lossy().to_string();
    // Keep the directory alive by preventing automatic cleanup
    std::mem::forget(temp_dir);
    Ok(path)
}

/// Unzip an FMU file to a temporary directory
pub fn unzip_fmu<P: AsRef<Path>>(fmu_path: P) -> Result<String> {
    use std::fs::File;
    use zip::ZipArchive;

    let file = File::open(fmu_path)?;
    let mut archive = ZipArchive::new(file)?;

    let output_dir = create_temp_directory()?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = Path::new(&output_dir).join(file.name());

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                std::fs::create_dir_all(p)?;
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(output_dir)
}

/// Add a file to a ZIP archive
pub fn add_file_to_fmu<P: AsRef<Path>>(
    zip: &mut zip::ZipWriter<std::fs::File>,
    file_path: P,
    archive_name: &str,
) -> Result<()> {
    use std::fs::File;
    use std::io::{Read, Write};
    use zip::write::SimpleFileOptions;

    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    zip.start_file(archive_name, SimpleFileOptions::default())?;
    zip.write_all(&buffer)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    // Helper function to create a test ZIP file
    fn create_test_fmu(temp_dir: &TempDir, name: &str) -> PathBuf {
        let fmu_path = temp_dir.path().join(name);
        let file = File::create(&fmu_path).unwrap();
        let mut zip = ZipWriter::new(file);

        // Add some test files
        zip.start_file("modelDescription.xml", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"<fmiModelDescription></fmiModelDescription>")
            .unwrap();

        zip.start_file("binaries/linux64/model.so", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"fake binary content").unwrap();

        zip.start_file("resources/data.txt", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"test data").unwrap();

        // Add a directory entry
        zip.add_directory("sources/", SimpleFileOptions::default())
            .unwrap();

        zip.finish().unwrap();
        fmu_path
    }

    #[test]
    fn test_create_directories_simple() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("test_dir");

        let result = create_directories(&test_path);
        assert!(result.is_ok());
        assert!(test_path.exists());
        assert!(test_path.is_dir());
    }

    #[test]
    fn test_create_directories_nested() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("a").join("b").join("c").join("d");

        let result = create_directories(&test_path);
        assert!(result.is_ok());
        assert!(test_path.exists());
        assert!(test_path.is_dir());
    }

    #[test]
    fn test_create_directories_already_exists() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("existing");

        // Create the directory first
        fs::create_dir(&test_path).unwrap();

        // Should succeed even if directory already exists
        let result = create_directories(&test_path);
        assert!(result.is_ok());
        assert!(test_path.exists());
    }

    #[test]
    fn test_create_directories_empty_path() {
        let result = create_directories("");
        // Should create current directory or succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_temp_directory_creates_valid_path() {
        let result = create_temp_directory();
        assert!(result.is_ok());

        let path_str = result.unwrap();
        assert!(!path_str.is_empty());

        let path = Path::new(&path_str);
        assert!(path.exists());
        assert!(path.is_dir());

        // Clean up
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn test_create_temp_directory_unique_paths() {
        let result1 = create_temp_directory();
        let result2 = create_temp_directory();

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        let path1 = result1.unwrap();
        let path2 = result2.unwrap();

        // Should create different directories
        assert_ne!(path1, path2);

        // Clean up
        let _ = fs::remove_dir_all(&path1);
        let _ = fs::remove_dir_all(&path2);
    }

    #[test]
    fn test_create_temp_directory_persists() {
        let path_str = create_temp_directory().unwrap();
        let path = Path::new(&path_str);

        // Create a file in the temp directory
        let test_file = path.join("test.txt");
        File::create(&test_file).unwrap();

        // File should exist
        assert!(test_file.exists());

        // Clean up
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn test_unzip_fmu_extracts_all_files() {
        let temp_dir = TempDir::new().unwrap();
        let fmu_path = create_test_fmu(&temp_dir, "test.fmu");

        let result = unzip_fmu(&fmu_path);
        assert!(result.is_ok());

        let output_dir = result.unwrap();
        let output_path = Path::new(&output_dir);

        // Verify extracted files exist
        assert!(output_path.join("modelDescription.xml").exists());
        assert!(output_path.join("binaries/linux64/model.so").exists());
        assert!(output_path.join("resources/data.txt").exists());
        assert!(output_path.join("sources").exists());
        assert!(output_path.join("sources").is_dir());

        // Clean up
        let _ = fs::remove_dir_all(output_path);
    }

    #[test]
    fn test_unzip_fmu_preserves_content() {
        let temp_dir = TempDir::new().unwrap();
        let fmu_path = create_test_fmu(&temp_dir, "test.fmu");

        let output_dir = unzip_fmu(&fmu_path).unwrap();
        let output_path = Path::new(&output_dir);

        // Verify content is preserved
        let xml_content = fs::read_to_string(output_path.join("modelDescription.xml")).unwrap();
        assert_eq!(xml_content, "<fmiModelDescription></fmiModelDescription>");

        let binary_content = fs::read(output_path.join("binaries/linux64/model.so")).unwrap();
        assert_eq!(binary_content, b"fake binary content");

        let data_content = fs::read_to_string(output_path.join("resources/data.txt")).unwrap();
        assert_eq!(data_content, "test data");

        // Clean up
        let _ = fs::remove_dir_all(output_path);
    }

    #[test]
    fn test_unzip_fmu_nonexistent_file() {
        let result = unzip_fmu("/nonexistent/path/to/file.fmu");
        assert!(result.is_err());
    }

    #[test]
    fn test_unzip_fmu_invalid_zip() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_fmu = temp_dir.path().join("invalid.fmu");

        // Create a file that's not a valid ZIP
        let mut file = File::create(&invalid_fmu).unwrap();
        file.write_all(b"This is not a ZIP file").unwrap();

        let result = unzip_fmu(&invalid_fmu);
        assert!(result.is_err());
    }

    #[test]
    fn test_unzip_fmu_empty_zip() {
        let temp_dir = TempDir::new().unwrap();
        let empty_fmu = temp_dir.path().join("empty.fmu");

        // Create an empty ZIP file
        let file = File::create(&empty_fmu).unwrap();
        let zip = ZipWriter::new(file);
        zip.finish().unwrap();

        let result = unzip_fmu(&empty_fmu);
        assert!(result.is_ok());

        let output_dir = result.unwrap();
        let output_path = Path::new(&output_dir);
        assert!(output_path.exists());

        // Clean up
        let _ = fs::remove_dir_all(output_path);
    }

    #[test]
    fn test_unzip_fmu_with_nested_directories() {
        let temp_dir = TempDir::new().unwrap();
        let fmu_path = temp_dir.path().join("nested.fmu");

        let file = File::create(&fmu_path).unwrap();
        let mut zip = ZipWriter::new(file);

        // Create deeply nested structure
        zip.start_file("a/b/c/d/e/file.txt", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"nested content").unwrap();

        zip.finish().unwrap();

        let result = unzip_fmu(&fmu_path);
        assert!(result.is_ok());

        let output_dir = result.unwrap();
        let output_path = Path::new(&output_dir);

        assert!(output_path.join("a/b/c/d/e/file.txt").exists());
        let content = fs::read_to_string(output_path.join("a/b/c/d/e/file.txt")).unwrap();
        assert_eq!(content, "nested content");

        // Clean up
        let _ = fs::remove_dir_all(output_path);
    }

    #[test]
    fn test_add_file_to_fmu_basic() {
        let temp_dir = TempDir::new().unwrap();

        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        let mut file = File::create(&test_file).unwrap();
        file.write_all(b"Hello, FMU!").unwrap();

        // Create a ZIP archive
        let zip_path = temp_dir.path().join("output.zip");
        let zip_file = File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(zip_file);

        // Add file to archive
        let result = add_file_to_fmu(&mut zip, &test_file, "test.txt");
        assert!(result.is_ok());

        zip.finish().unwrap();

        // Verify the file was added
        let file = File::open(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        assert_eq!(archive.len(), 1);

        let mut archived_file = archive.by_name("test.txt").unwrap();
        let mut content = String::new();
        std::io::Read::read_to_string(&mut archived_file, &mut content).unwrap();
        assert_eq!(content, "Hello, FMU!");
    }

    #[test]
    fn test_add_file_to_fmu_with_path() {
        let temp_dir = TempDir::new().unwrap();

        // Create a test file
        let test_file = temp_dir.path().join("source.txt");
        let mut file = File::create(&test_file).unwrap();
        file.write_all(b"Content").unwrap();

        // Create a ZIP archive
        let zip_path = temp_dir.path().join("output.zip");
        let zip_file = File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(zip_file);

        // Add file with a different archive name including path
        let result = add_file_to_fmu(&mut zip, &test_file, "resources/data/source.txt");
        assert!(result.is_ok());

        zip.finish().unwrap();

        // Verify the file was added with correct path
        let file = File::open(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        let mut archived_file = archive.by_name("resources/data/source.txt").unwrap();
        let mut content = String::new();
        std::io::Read::read_to_string(&mut archived_file, &mut content).unwrap();
        assert_eq!(content, "Content");
    }

    #[test]
    fn test_add_file_to_fmu_multiple_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        File::create(&file1).unwrap().write_all(b"First").unwrap();
        File::create(&file2).unwrap().write_all(b"Second").unwrap();

        // Create a ZIP archive
        let zip_path = temp_dir.path().join("output.zip");
        let zip_file = File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(zip_file);

        // Add multiple files
        assert!(add_file_to_fmu(&mut zip, &file1, "file1.txt").is_ok());
        assert!(add_file_to_fmu(&mut zip, &file2, "file2.txt").is_ok());

        zip.finish().unwrap();

        // Verify both files were added
        let file = File::open(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        assert_eq!(archive.len(), 2);
        assert!(archive.by_name("file1.txt").is_ok());
        assert!(archive.by_name("file2.txt").is_ok());
    }

    #[test]
    fn test_add_file_to_fmu_binary_content() {
        let temp_dir = TempDir::new().unwrap();

        // Create a binary file
        let binary_file = temp_dir.path().join("binary.bin");
        let binary_data: Vec<u8> = (0..=255).collect();
        File::create(&binary_file)
            .unwrap()
            .write_all(&binary_data)
            .unwrap();

        // Create a ZIP archive
        let zip_path = temp_dir.path().join("output.zip");
        let zip_file = File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(zip_file);

        // Add binary file
        let result = add_file_to_fmu(&mut zip, &binary_file, "binary.bin");
        assert!(result.is_ok());

        zip.finish().unwrap();

        // Verify binary content is preserved
        let file = File::open(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        let mut archived_file = archive.by_name("binary.bin").unwrap();
        let mut content = Vec::new();
        std::io::Read::read_to_end(&mut archived_file, &mut content).unwrap();
        assert_eq!(content, binary_data);
    }

    #[test]
    fn test_add_file_to_fmu_empty_file() {
        let temp_dir = TempDir::new().unwrap();

        // Create an empty file
        let empty_file = temp_dir.path().join("empty.txt");
        File::create(&empty_file).unwrap();

        // Create a ZIP archive
        let zip_path = temp_dir.path().join("output.zip");
        let zip_file = File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(zip_file);

        // Add empty file
        let result = add_file_to_fmu(&mut zip, &empty_file, "empty.txt");
        assert!(result.is_ok());

        zip.finish().unwrap();

        // Verify empty file was added
        let file = File::open(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        let archived_file = archive.by_name("empty.txt").unwrap();
        assert_eq!(archived_file.size(), 0);
    }

    #[test]
    fn test_add_file_to_fmu_large_file() {
        let temp_dir = TempDir::new().unwrap();

        // Create a large file (1MB)
        let large_file = temp_dir.path().join("large.txt");
        let large_data = vec![b'X'; 1024 * 1024];
        File::create(&large_file)
            .unwrap()
            .write_all(&large_data)
            .unwrap();

        // Create a ZIP archive
        let zip_path = temp_dir.path().join("output.zip");
        let zip_file = File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(zip_file);

        // Add large file
        let result = add_file_to_fmu(&mut zip, &large_file, "large.txt");
        assert!(result.is_ok());

        zip.finish().unwrap();

        // Verify large file was added and content matches
        let file = File::open(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        let mut archived_file = archive.by_name("large.txt").unwrap();
        let mut content = Vec::new();
        std::io::Read::read_to_end(&mut archived_file, &mut content).unwrap();
        assert_eq!(content.len(), large_data.len());
        assert_eq!(content, large_data);
    }

    #[test]
    fn test_add_file_to_fmu_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();

        let zip_path = temp_dir.path().join("output.zip");
        let zip_file = File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(zip_file);

        // Try to add a nonexistent file
        let result = add_file_to_fmu(&mut zip, "/nonexistent/file.txt", "file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_add_file_to_fmu_special_characters_in_name() {
        let temp_dir = TempDir::new().unwrap();

        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        File::create(&test_file)
            .unwrap()
            .write_all(b"Test")
            .unwrap();

        // Create a ZIP archive
        let zip_path = temp_dir.path().join("output.zip");
        let zip_file = File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(zip_file);

        // Add file with special characters in archive name
        let result = add_file_to_fmu(&mut zip, &test_file, "files/test-file_v2.0.txt");
        assert!(result.is_ok());

        zip.finish().unwrap();

        // Verify the file was added
        let file = File::open(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        assert!(archive.by_name("files/test-file_v2.0.txt").is_ok());
    }
}
