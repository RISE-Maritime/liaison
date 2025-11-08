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
    use zip::ZipArchive;
    use std::fs::File;

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
