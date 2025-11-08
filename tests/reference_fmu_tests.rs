// Integration tests for Liaison FMU creation with reference FMUs
//
// These tests verify the complete FMU creation workflow:
// 1. Creating a Liaison FMU using the make-fmu command
// 2. Validating the structure and content of the created FMU
// 3. Ensuring all required files are present and correctly formatted
//
// Test strategy:
// - Use real reference FMUs (BouncingBall.fmu)
// - Run the liaison binary programmatically
// - Extract and inspect the created FMU
// - Validate all components (modelDescription.xml, config.json, binaries)

use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;
use zip::ZipArchive;

/// Helper function to find the workspace directory
fn get_workspace_dir() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;

    // Look for Cargo.toml with [workspace]
    let mut dir = current_dir.as_path();
    loop {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = std::fs::read_to_string(&cargo_toml)?;
            if content.contains("[workspace]") {
                return Ok(dir.to_path_buf());
            }
        }

        match dir.parent() {
            Some(parent) => dir = parent,
            None => anyhow::bail!("Could not find workspace root"),
        }
    }
}

/// Test fixture that manages test environment and cleanup
struct FmuTestFixture {
    temp_dir: TempDir,
    workspace_dir: PathBuf,
    binaries_dir: PathBuf,
}

impl FmuTestFixture {
    /// Create a new test fixture with temporary directories
    fn new() -> Result<Self> {
        let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
        let workspace_dir = get_workspace_dir()?;
        let binaries_dir = temp_dir.path().join("binaries");

        Ok(FmuTestFixture {
            temp_dir,
            workspace_dir,
            binaries_dir,
        })
    }

    /// Set up the binaries directory structure required for FMU creation
    fn setup_binaries(&self) -> Result<()> {
        // Create platform-specific directories
        let linux_dir = self.binaries_dir.join("x86_64-linux");
        let windows_dir = self.binaries_dir.join("x86_64-windows");

        fs::create_dir_all(&linux_dir)
            .context("Failed to create Linux binaries directory")?;
        fs::create_dir_all(&windows_dir)
            .context("Failed to create Windows binaries directory")?;

        // Copy the built liaison FMU library to the binaries directory
        let source_lib = self.workspace_dir.join("target/release/libliaisonfmu.so");
        let dest_lib = linux_dir.join("libliaisonfmu.so");

        if !source_lib.exists() {
            anyhow::bail!(
                "Liaison FMU library not found at {}. Please run 'cargo build --release' first.",
                source_lib.display()
            );
        }

        fs::copy(&source_lib, &dest_lib)
            .context("Failed to copy liaison FMU library to binaries directory")?;

        tracing::info!(
            "Set up binaries directory at: {}",
            self.binaries_dir.display()
        );
        tracing::info!("  Linux library: {}", dest_lib.display());

        Ok(())
    }

    /// Get the path to the liaison binary
    fn get_liaison_binary(&self) -> PathBuf {
        self.workspace_dir.join("target/release/liaison")
    }

    /// Get the path to a reference FMU
    fn get_reference_fmu(&self, name: &str) -> PathBuf {
        self.workspace_dir.join("examples").join(name)
    }

    /// Run the liaison make-fmu command
    fn make_fmu(
        &self,
        source_fmu: &Path,
        responder_id: &str,
        zenoh_config: Option<&Path>,
    ) -> Result<PathBuf> {
        let liaison_binary = self.get_liaison_binary();

        if !liaison_binary.exists() {
            anyhow::bail!(
                "Liaison binary not found at {}. Please run 'cargo build --release' first.",
                liaison_binary.display()
            );
        }

        let mut cmd = Command::new(&liaison_binary);
        cmd.arg("make-fmu")
            .arg(source_fmu)
            .arg(responder_id)
            .current_dir(self.temp_dir.path());

        if let Some(config) = zenoh_config {
            cmd.arg("--zenoh-config").arg(config);
        }

        let output = cmd.output().context("Failed to run liaison make-fmu")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            anyhow::bail!(
                "liaison make-fmu failed:\nStdout: {}\nStderr: {}",
                stdout,
                stderr
            );
        }

        // The output FMU should be named {modelName}Liaison.fmu
        let model_name = source_fmu
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("Failed to extract model name"))?;

        let output_fmu = self.temp_dir.path().join(format!("{}Liaison.fmu", model_name));

        if !output_fmu.exists() {
            anyhow::bail!(
                "Expected output FMU not found at: {}",
                output_fmu.display()
            );
        }

        Ok(output_fmu)
    }

    /// Extract an FMU to a temporary directory
    fn extract_fmu(&self, fmu_path: &Path) -> Result<PathBuf> {
        let extract_dir = self.temp_dir.path().join("extracted");
        fs::create_dir_all(&extract_dir)?;

        let file = File::open(fmu_path)
            .context(format!("Failed to open FMU file: {}", fmu_path.display()))?;
        let mut archive = ZipArchive::new(file).context("Failed to read FMU as ZIP archive")?;

        archive
            .extract(&extract_dir)
            .context("Failed to extract FMU")?;

        Ok(extract_dir)
    }
}

// ============================================================================
// Test Cases
// ============================================================================

#[test]
fn test_create_liaison_fmu_basic() -> Result<()> {
    // Initialize tracing for test output
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init();

    let fixture = FmuTestFixture::new()?;
    fixture.setup_binaries()?;

    // Use the BouncingBall reference FMU
    let source_fmu = fixture.get_reference_fmu("BouncingBall.fmu");
    if !source_fmu.exists() {
        eprintln!(
            "Reference FMU not found at: {}. Skipping test.",
            source_fmu.display()
        );
        return Ok(());
    }

    // Create the Liaison FMU
    let output_fmu = fixture.make_fmu(&source_fmu, "test-bouncing-ball", None)?;

    // Verify the output FMU exists
    assert!(
        output_fmu.exists(),
        "Output FMU should exist at {}",
        output_fmu.display()
    );

    // Verify it's a valid ZIP file
    let fmu_file = File::open(&output_fmu)?;
    let archive = ZipArchive::new(fmu_file);
    assert!(
        archive.is_ok(),
        "Output FMU should be a valid ZIP archive"
    );

    tracing::info!("Successfully created Liaison FMU: {}", output_fmu.display());

    Ok(())
}

#[test]
fn test_liaison_fmu_structure() -> Result<()> {
    // Initialize tracing for test output
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init();

    let fixture = FmuTestFixture::new()?;
    fixture.setup_binaries()?;

    // Use the BouncingBall reference FMU
    let source_fmu = fixture.get_reference_fmu("BouncingBall.fmu");
    if !source_fmu.exists() {
        eprintln!(
            "Reference FMU not found at: {}. Skipping test.",
            source_fmu.display()
        );
        return Ok(());
    }

    // Create the Liaison FMU
    let output_fmu = fixture.make_fmu(&source_fmu, "test-structure", None)?;

    // Extract the FMU
    let extract_dir = fixture.extract_fmu(&output_fmu)?;

    // Validate required files exist
    let required_files = vec![
        "modelDescription.xml",
        "binaries/config.json",
        "binaries/x86_64-linux/BouncingBall.so",
    ];

    for file in required_files {
        let file_path = extract_dir.join(file);
        assert!(
            file_path.exists(),
            "Required file '{}' should exist in the FMU",
            file
        );
        tracing::info!("Verified file exists: {}", file);
    }

    Ok(())
}

#[test]
fn test_model_description_copied_correctly() -> Result<()> {
    // Initialize tracing for test output
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init();

    let fixture = FmuTestFixture::new()?;
    fixture.setup_binaries()?;

    // Use the BouncingBall reference FMU
    let source_fmu = fixture.get_reference_fmu("BouncingBall.fmu");
    if !source_fmu.exists() {
        eprintln!(
            "Reference FMU not found at: {}. Skipping test.",
            source_fmu.display()
        );
        return Ok(());
    }

    // Extract the source FMU to read its modelDescription.xml
    let source_extract_dir = fixture.extract_fmu(&source_fmu)?;
    let source_model_desc_path = source_extract_dir.join("modelDescription.xml");
    let mut source_model_desc = String::new();
    File::open(&source_model_desc_path)?
        .read_to_string(&mut source_model_desc)?;

    // Create the Liaison FMU
    let output_fmu = fixture.make_fmu(&source_fmu, "test-model-desc", None)?;

    // Extract the output FMU
    let output_extract_dir = fixture.extract_fmu(&output_fmu)?;
    let output_model_desc_path = output_extract_dir.join("modelDescription.xml");
    let mut output_model_desc = String::new();
    File::open(&output_model_desc_path)?
        .read_to_string(&mut output_model_desc)?;

    // Verify modelDescription.xml was copied correctly
    assert_eq!(
        source_model_desc, output_model_desc,
        "modelDescription.xml should be identical to the source FMU"
    );

    // Verify it contains expected FMI 3.0 elements
    assert!(
        output_model_desc.contains("fmiVersion=\"3.0\""),
        "modelDescription.xml should specify FMI version 3.0"
    );
    assert!(
        output_model_desc.contains("modelName=\"BouncingBall\""),
        "modelDescription.xml should contain the model name"
    );

    tracing::info!("modelDescription.xml verified successfully");

    Ok(())
}

#[test]
fn test_config_json_embedded_correctly() -> Result<()> {
    // Initialize tracing for test output
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init();

    let fixture = FmuTestFixture::new()?;
    fixture.setup_binaries()?;

    // Use the BouncingBall reference FMU
    let source_fmu = fixture.get_reference_fmu("BouncingBall.fmu");
    if !source_fmu.exists() {
        eprintln!(
            "Reference FMU not found at: {}. Skipping test.",
            source_fmu.display()
        );
        return Ok(());
    }

    let responder_id = "test-config-json";

    // Create the Liaison FMU
    let output_fmu = fixture.make_fmu(&source_fmu, responder_id, None)?;

    // Extract the output FMU
    let extract_dir = fixture.extract_fmu(&output_fmu)?;
    let config_path = extract_dir.join("binaries/config.json");

    // Read and parse config.json
    let config_content = fs::read_to_string(&config_path)
        .context("Failed to read config.json from FMU")?;
    let config: serde_json::Value = serde_json::from_str(&config_content)
        .context("Failed to parse config.json")?;

    // Verify config.json structure
    assert!(
        config.is_object(),
        "config.json should be a JSON object"
    );

    // Verify responderId is correct
    let responder_id_value = config
        .get("responderId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("config.json should contain 'responderId' field"))?;
    assert_eq!(
        responder_id_value, responder_id,
        "responderId in config.json should match the specified value"
    );

    // Verify name field exists
    let name_value = config
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("config.json should contain 'name' field"))?;
    assert_eq!(
        name_value, "BouncingBall",
        "name should match the model name from the FMU"
    );

    tracing::info!("config.json verified successfully");
    tracing::info!("  responderId: {}", responder_id_value);
    tracing::info!("  name: {}", name_value);

    Ok(())
}

#[test]
fn test_liaison_library_included_in_fmu() -> Result<()> {
    // Initialize tracing for test output
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init();

    let fixture = FmuTestFixture::new()?;
    fixture.setup_binaries()?;

    // Use the BouncingBall reference FMU
    let source_fmu = fixture.get_reference_fmu("BouncingBall.fmu");
    if !source_fmu.exists() {
        eprintln!(
            "Reference FMU not found at: {}. Skipping test.",
            source_fmu.display()
        );
        return Ok(());
    }

    // Create the Liaison FMU
    let output_fmu = fixture.make_fmu(&source_fmu, "test-library", None)?;

    // Extract the output FMU
    let extract_dir = fixture.extract_fmu(&output_fmu)?;

    // Check for Linux library
    let linux_lib = extract_dir.join("binaries/x86_64-linux/BouncingBall.so");
    assert!(
        linux_lib.exists(),
        "Linux liaison library should be included at binaries/x86_64-linux/BouncingBall.so"
    );

    // Verify the library file has non-zero size
    let lib_metadata = fs::metadata(&linux_lib)?;
    assert!(
        lib_metadata.len() > 0,
        "Liaison library should have non-zero size"
    );

    tracing::info!(
        "Liaison library verified: {} ({} bytes)",
        linux_lib.display(),
        lib_metadata.len()
    );

    // Verify it's an ELF file (Linux shared library)
    let mut lib_file = File::open(&linux_lib)?;
    let mut magic = [0u8; 4];
    lib_file.read_exact(&mut magic)?;

    assert_eq!(
        &magic,
        b"\x7fELF",
        "Library should be a valid ELF file (Linux shared library)"
    );

    tracing::info!("Verified library is a valid ELF file");

    Ok(())
}

#[test]
fn test_liaison_fmu_with_zenoh_config() -> Result<()> {
    // Initialize tracing for test output
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init();

    let fixture = FmuTestFixture::new()?;
    fixture.setup_binaries()?;

    // Create a Zenoh config file
    let zenoh_config_path = fixture.temp_dir.path().join("zenoh.json5");
    let config_content = serde_json::json!({
        "mode": "peer",
        "connect": {
            "endpoints": ["tcp/localhost:7447"]
        }
    });
    fs::write(
        &zenoh_config_path,
        serde_json::to_string_pretty(&config_content)?,
    )?;

    // Use the BouncingBall reference FMU
    let source_fmu = fixture.get_reference_fmu("BouncingBall.fmu");
    if !source_fmu.exists() {
        eprintln!(
            "Reference FMU not found at: {}. Skipping test.",
            source_fmu.display()
        );
        return Ok(());
    }

    // Create the Liaison FMU with Zenoh config
    let output_fmu = fixture.make_fmu(&source_fmu, "test-zenoh", Some(&zenoh_config_path))?;

    // Extract the output FMU
    let extract_dir = fixture.extract_fmu(&output_fmu)?;

    // Read config.json which should contain embedded Zenoh config
    let config_path = extract_dir.join("binaries/config.json");
    let config_content = fs::read_to_string(&config_path)?;
    let config: serde_json::Value = serde_json::from_str(&config_content)?;

    // Verify zenohConfig field exists and contains the configuration
    let zenoh_config_field = config
        .get("zenohConfig")
        .ok_or_else(|| anyhow::anyhow!("config.json should contain 'zenohConfig' field"))?;

    // Verify the Zenoh config has expected fields
    assert!(
        zenoh_config_field.is_object(),
        "zenohConfig should be a JSON object"
    );

    assert_eq!(
        zenoh_config_field.get("mode").and_then(|v| v.as_str()),
        Some("peer"),
        "Zenoh config should contain mode: peer"
    );

    // Verify metadata.name was set to the model name
    let metadata_name = zenoh_config_field
        .get("metadata")
        .and_then(|m| m.get("name"))
        .and_then(|n| n.as_str());
    assert_eq!(
        metadata_name,
        Some("BouncingBall"),
        "Zenoh config metadata.name should be set to model name"
    );

    tracing::info!("Zenoh configuration verified successfully");
    tracing::info!("  Mode: peer");
    tracing::info!("  Metadata name: {}", metadata_name.unwrap());

    Ok(())
}

#[test]
fn test_multiple_fmu_creation() -> Result<()> {
    // Initialize tracing for test output
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init();

    let fixture = FmuTestFixture::new()?;
    fixture.setup_binaries()?;

    // Test with multiple reference FMUs if available
    let reference_fmus = vec!["BouncingBall.fmu", "BouncingBallPython.fmu"];

    for (idx, fmu_name) in reference_fmus.iter().enumerate() {
        let source_fmu = fixture.get_reference_fmu(fmu_name);
        if !source_fmu.exists() {
            eprintln!(
                "Reference FMU not found at: {}. Skipping.",
                source_fmu.display()
            );
            continue;
        }

        let responder_id = format!("test-multiple-{}", idx);
        let output_fmu = fixture.make_fmu(&source_fmu, &responder_id, None)?;

        assert!(
            output_fmu.exists(),
            "Output FMU should exist for {}",
            fmu_name
        );

        tracing::info!("Successfully created FMU {}: {}", idx + 1, fmu_name);
    }

    Ok(())
}

#[test]
fn test_fmu_zip_structure_integrity() -> Result<()> {
    // Initialize tracing for test output
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init();

    let fixture = FmuTestFixture::new()?;
    fixture.setup_binaries()?;

    // Use the BouncingBall reference FMU
    let source_fmu = fixture.get_reference_fmu("BouncingBall.fmu");
    if !source_fmu.exists() {
        eprintln!(
            "Reference FMU not found at: {}. Skipping test.",
            source_fmu.display()
        );
        return Ok(());
    }

    // Create the Liaison FMU
    let output_fmu = fixture.make_fmu(&source_fmu, "test-zip-structure", None)?;

    // Open and inspect the ZIP archive
    let file = File::open(&output_fmu)?;
    let mut archive = ZipArchive::new(file)?;

    tracing::info!("FMU contains {} files:", archive.len());

    let mut found_files = std::collections::HashSet::new();

    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        let file_name = file.name().to_string();
        tracing::info!("  {}: {} ({} bytes)", i, file_name, file.size());
        found_files.insert(file_name);
    }

    // Verify essential files are present
    assert!(
        found_files.contains("modelDescription.xml"),
        "FMU must contain modelDescription.xml"
    );
    assert!(
        found_files.contains("binaries/config.json"),
        "FMU must contain binaries/config.json"
    );
    assert!(
        found_files
            .iter()
            .any(|f| f.starts_with("binaries/") && f.ends_with(".so")),
        "FMU must contain at least one binary library"
    );

    tracing::info!("ZIP structure integrity verified");

    Ok(())
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_make_fmu_with_nonexistent_source() -> Result<()> {
    let fixture = FmuTestFixture::new()?;
    fixture.setup_binaries()?;

    let nonexistent_fmu = fixture.temp_dir.path().join("NonExistent.fmu");

    let result = fixture.make_fmu(&nonexistent_fmu, "test-error", None);

    assert!(
        result.is_err(),
        "Creating FMU from nonexistent source should fail"
    );

    Ok(())
}

#[test]
fn test_make_fmu_without_binaries_directory() -> Result<()> {
    let fixture = FmuTestFixture::new()?;
    // Note: We deliberately do NOT call setup_binaries() here

    let source_fmu = fixture.get_reference_fmu("BouncingBall.fmu");
    if !source_fmu.exists() {
        eprintln!(
            "Reference FMU not found at: {}. Skipping test.",
            source_fmu.display()
        );
        return Ok(());
    }

    let result = fixture.make_fmu(&source_fmu, "test-no-binaries", None);

    assert!(
        result.is_err(),
        "Creating FMU without binaries directory should fail"
    );

    if let Err(e) = result {
        let error_msg = format!("{}", e);
        assert!(
            error_msg.contains("binaries") || error_msg.contains("directory"),
            "Error message should mention missing binaries directory"
        );
    }

    Ok(())
}
