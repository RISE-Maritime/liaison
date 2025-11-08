// Tests for validating FMU structure and contents
//
// This test suite validates that the created Liaison FMU:
// 1. Has a valid FMI 3.0 structure
// 2. Contains all required files (modelDescription.xml, binary, config.json)
// 3. Has valid XML conforming to FMI 3.0 specification
// 4. Contains a valid ELF binary
// 5. Has properly structured configuration

use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
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

/// Test fixture that extracts and validates an FMU
struct FmuValidator {
    temp_dir: TempDir,
    fmu_path: std::path::PathBuf,
}

impl FmuValidator {
    /// Create a new FMU validator for the given FMU file
    fn new(fmu_path: impl AsRef<Path>) -> Result<Self> {
        let temp_dir = TempDir::new().context("Failed to create temp directory")?;
        let fmu_path = fmu_path.as_ref().to_path_buf();

        if !fmu_path.exists() {
            anyhow::bail!("FMU file does not exist: {}", fmu_path.display());
        }

        Ok(FmuValidator { temp_dir, fmu_path })
    }

    /// Extract the FMU to the temporary directory
    fn extract(&self) -> Result<std::path::PathBuf> {
        let file = fs::File::open(&self.fmu_path)
            .context("Failed to open FMU file")?;
        let mut archive = ZipArchive::new(file)
            .context("Failed to read FMU as ZIP archive")?;

        archive
            .extract(self.temp_dir.path())
            .context("Failed to extract FMU")?;

        Ok(self.temp_dir.path().to_path_buf())
    }

    /// Get the path to modelDescription.xml
    fn model_description_path(&self) -> std::path::PathBuf {
        self.temp_dir.path().join("modelDescription.xml")
    }

    /// Get the path to config.json
    fn config_json_path(&self) -> std::path::PathBuf {
        self.temp_dir.path().join("binaries/config.json")
    }

    /// Get the path to the binary directory
    fn binaries_path(&self) -> std::path::PathBuf {
        self.temp_dir.path().join("binaries")
    }
}

#[test]
fn test_fmu_exists() -> Result<()> {
    let fmu_path = get_workspace_dir()?.join("BouncingBallLiaison.fmu");

    assert!(
        fmu_path.exists(),
        "FMU file does not exist at {}. Run the FMU creation first.",
        fmu_path.display()
    );

    // Verify it's a valid ZIP file
    let file = fs::File::open(&fmu_path)?;
    let archive = ZipArchive::new(file);
    assert!(
        archive.is_ok(),
        "FMU file is not a valid ZIP archive"
    );

    println!("FMU file exists and is a valid ZIP archive");
    Ok(())
}

#[test]
fn test_fmu_directory_structure() -> Result<()> {
    let fmu_path = get_workspace_dir()?.join("BouncingBallLiaison.fmu");
    let validator = FmuValidator::new(fmu_path)?;
    validator.extract()?;

    // Check required files exist
    let model_desc = validator.model_description_path();
    assert!(
        model_desc.exists(),
        "modelDescription.xml not found in FMU"
    );

    let config_json = validator.config_json_path();
    assert!(
        config_json.exists(),
        "config.json not found in binaries directory"
    );

    let binaries = validator.binaries_path();
    assert!(
        binaries.exists() && binaries.is_dir(),
        "binaries directory not found"
    );

    // Check for platform-specific binary
    let binary_path = binaries.join("x86_64-linux/BouncingBall.so");
    assert!(
        binary_path.exists(),
        "Binary not found at expected location: {}",
        binary_path.display()
    );

    println!("FMU has correct directory structure:");
    println!("  - modelDescription.xml: OK");
    println!("  - binaries/config.json: OK");
    println!("  - binaries/x86_64-linux/BouncingBall.so: OK");

    Ok(())
}

#[test]
fn test_model_description_is_valid_xml() -> Result<()> {
    let fmu_path = get_workspace_dir()?.join("BouncingBallLiaison.fmu");
    let validator = FmuValidator::new(fmu_path)?;
    validator.extract()?;

    let xml_content = fs::read_to_string(validator.model_description_path())
        .context("Failed to read modelDescription.xml")?;

    // Parse XML
    let mut reader = Reader::from_str(&xml_content);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut found_root = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.name().as_ref() == b"fmiModelDescription" {
                    found_root = true;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                anyhow::bail!("Error parsing XML at position {}: {:?}", reader.buffer_position(), e);
            }
            _ => {}
        }
        buf.clear();
    }

    assert!(found_root, "fmiModelDescription root element not found");
    println!("modelDescription.xml is valid XML");

    Ok(())
}

#[test]
fn test_model_description_fmi_version() -> Result<()> {
    let fmu_path = get_workspace_dir()?.join("BouncingBallLiaison.fmu");
    let validator = FmuValidator::new(fmu_path)?;
    validator.extract()?;

    let xml_content = fs::read_to_string(validator.model_description_path())
        .context("Failed to read modelDescription.xml")?;

    // Parse XML and extract fmiVersion attribute
    let mut reader = Reader::from_str(&xml_content);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut fmi_version = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"fmiModelDescription" => {
                for attr in e.attributes() {
                    let attr = attr?;
                    if attr.key.as_ref() == b"fmiVersion" {
                        fmi_version = Some(String::from_utf8(attr.value.to_vec())?);
                    }
                }
                break;
            }
            Ok(Event::Eof) => break,
            Err(e) => anyhow::bail!("Error parsing XML: {:?}", e),
            _ => {}
        }
        buf.clear();
    }

    assert_eq!(
        fmi_version.as_deref(),
        Some("3.0"),
        "FMI version should be 3.0"
    );

    println!("FMI version is 3.0");
    Ok(())
}

#[test]
fn test_model_description_required_elements() -> Result<()> {
    let fmu_path = get_workspace_dir()?.join("BouncingBallLiaison.fmu");
    let validator = FmuValidator::new(fmu_path)?;
    validator.extract()?;

    let xml_content = fs::read_to_string(validator.model_description_path())
        .context("Failed to read modelDescription.xml")?;

    // Parse XML and check for required elements
    let mut reader = Reader::from_str(&xml_content);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut attributes = HashMap::new();
    let mut found_elements = HashMap::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let name = String::from_utf8(e.name().as_ref().to_vec())?;

                if name == "fmiModelDescription" {
                    // Extract key attributes from root element
                    for attr in e.attributes() {
                        let attr = attr?;
                        let key = String::from_utf8(attr.key.as_ref().to_vec())?;
                        let value = String::from_utf8(attr.value.to_vec())?;
                        attributes.insert(key, value);
                    }
                }

                found_elements.insert(name, true);
            }
            Ok(Event::Eof) => break,
            Err(e) => anyhow::bail!("Error parsing XML: {:?}", e),
            _ => {}
        }
        buf.clear();
    }

    // Check required attributes
    assert!(
        attributes.contains_key("fmiVersion"),
        "fmiVersion attribute missing"
    );
    assert!(
        attributes.contains_key("modelName"),
        "modelName attribute missing"
    );
    assert!(
        attributes.contains_key("instantiationToken"),
        "instantiationToken attribute missing"
    );

    // Check model name
    assert_eq!(
        attributes.get("modelName").map(|s| s.as_str()),
        Some("BouncingBall"),
        "modelName should be 'BouncingBall'"
    );

    // Check required elements for FMI 3.0 Co-Simulation
    assert!(
        found_elements.contains_key("CoSimulation"),
        "CoSimulation element missing"
    );
    assert!(
        found_elements.contains_key("ModelVariables"),
        "ModelVariables element missing"
    );

    println!("modelDescription.xml contains all required FMI 3.0 elements:");
    println!("  - fmiVersion: {}", attributes.get("fmiVersion").unwrap());
    println!("  - modelName: {}", attributes.get("modelName").unwrap());
    println!("  - instantiationToken: {}", attributes.get("instantiationToken").unwrap());
    println!("  - CoSimulation: OK");
    println!("  - ModelVariables: OK");

    Ok(())
}

#[test]
fn test_binary_is_valid_elf() -> Result<()> {
    let fmu_path = get_workspace_dir()?.join("BouncingBallLiaison.fmu");
    let validator = FmuValidator::new(fmu_path)?;
    validator.extract()?;

    let binary_path = validator.binaries_path().join("x86_64-linux/BouncingBall.so");

    // Read first 4 bytes to check ELF magic number
    let mut file = fs::File::open(&binary_path)
        .context("Failed to open binary file")?;

    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)
        .context("Failed to read binary magic number")?;

    // ELF magic number is 0x7f 'E' 'L' 'F'
    assert_eq!(
        magic,
        [0x7f, 0x45, 0x4c, 0x46],
        "Binary is not a valid ELF file (magic number mismatch)"
    );

    println!("Binary is a valid ELF file (magic: 7f 45 4c 46)");

    // Additional check: verify it's a 64-bit shared object
    let mut class_and_data = [0u8; 2];
    file.read_exact(&mut class_and_data)
        .context("Failed to read ELF class")?;

    // class_and_data[0] should be 2 for 64-bit
    assert_eq!(
        class_and_data[0], 2,
        "Binary should be 64-bit (ELF class 2)"
    );

    println!("Binary is 64-bit ELF");

    Ok(())
}

#[test]
fn test_config_json_is_valid() -> Result<()> {
    let fmu_path = get_workspace_dir()?.join("BouncingBallLiaison.fmu");
    let validator = FmuValidator::new(fmu_path)?;
    validator.extract()?;

    let config_content = fs::read_to_string(validator.config_json_path())
        .context("Failed to read config.json")?;

    // Parse JSON
    let config: serde_json::Value = serde_json::from_str(&config_content)
        .context("Failed to parse config.json")?;

    // Check required fields
    assert!(
        config.get("name").is_some(),
        "config.json missing 'name' field"
    );
    assert!(
        config.get("responderId").is_some(),
        "config.json missing 'responderId' field"
    );

    // Verify name matches model
    assert_eq!(
        config.get("name").and_then(|v| v.as_str()),
        Some("BouncingBall"),
        "config.json 'name' should be 'BouncingBall'"
    );

    println!("config.json is valid:");
    println!("  - name: {}", config["name"]);
    println!("  - responderId: {}", config["responderId"]);

    Ok(())
}

#[test]
fn test_fmu_complete_validation() -> Result<()> {
    println!("\n=== Complete FMU Validation ===\n");

    let fmu_path = get_workspace_dir()?.join("BouncingBallLiaison.fmu");
    let validator = FmuValidator::new(&fmu_path)?;
    validator.extract()?;

    let mut checks_passed = 0;
    let mut total_checks = 0;

    // Check 1: ZIP archive integrity
    total_checks += 1;
    let file = fs::File::open(&fmu_path)?;
    match ZipArchive::new(file) {
        Ok(_) => {
            println!("[PASS] FMU is a valid ZIP archive");
            checks_passed += 1;
        }
        Err(e) => println!("[FAIL] ZIP archive validation: {}", e),
    }

    // Check 2: Directory structure
    total_checks += 1;
    if validator.model_description_path().exists()
        && validator.config_json_path().exists()
        && validator.binaries_path().join("x86_64-linux/BouncingBall.so").exists()
    {
        println!("[PASS] FMU directory structure is correct");
        checks_passed += 1;
    } else {
        println!("[FAIL] FMU directory structure incomplete");
    }

    // Check 3: XML validity
    total_checks += 1;
    let xml_content = fs::read_to_string(validator.model_description_path())?;
    let mut reader = Reader::from_str(&xml_content);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut xml_valid = true;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Err(_) => {
                xml_valid = false;
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    if xml_valid {
        println!("[PASS] modelDescription.xml is valid XML");
        checks_passed += 1;
    } else {
        println!("[FAIL] modelDescription.xml has XML errors");
    }

    // Check 4: FMI version
    total_checks += 1;
    if xml_content.contains("fmiVersion=\"3.0\"") {
        println!("[PASS] FMI version is 3.0");
        checks_passed += 1;
    } else {
        println!("[FAIL] FMI version is not 3.0");
    }

    // Check 5: Model name
    total_checks += 1;
    if xml_content.contains("modelName=\"BouncingBall\"") {
        println!("[PASS] Model name is correct");
        checks_passed += 1;
    } else {
        println!("[FAIL] Model name is incorrect");
    }

    // Check 6: ELF binary
    total_checks += 1;
    let binary_path = validator.binaries_path().join("x86_64-linux/BouncingBall.so");
    let mut file = fs::File::open(&binary_path)?;
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;

    if magic == [0x7f, 0x45, 0x4c, 0x46] {
        println!("[PASS] Binary is a valid ELF file");
        checks_passed += 1;
    } else {
        println!("[FAIL] Binary is not a valid ELF file");
    }

    // Check 7: Config JSON
    total_checks += 1;
    let config_content = fs::read_to_string(validator.config_json_path())?;
    match serde_json::from_str::<serde_json::Value>(&config_content) {
        Ok(config) => {
            if config.get("name").is_some() && config.get("responderId").is_some() {
                println!("[PASS] config.json is valid and complete");
                checks_passed += 1;
            } else {
                println!("[FAIL] config.json missing required fields");
            }
        }
        Err(_) => println!("[FAIL] config.json is not valid JSON"),
    }

    // Summary
    println!("\n=== Validation Summary ===");
    println!("Checks passed: {}/{}", checks_passed, total_checks);
    println!("Status: {}", if checks_passed == total_checks { "PASS" } else { "FAIL" });

    assert_eq!(
        checks_passed, total_checks,
        "Not all validation checks passed"
    );

    Ok(())
}

#[test]
fn test_fmu_model_variables_present() -> Result<()> {
    let fmu_path = get_workspace_dir()?.join("BouncingBallLiaison.fmu");
    let validator = FmuValidator::new(fmu_path)?;
    validator.extract()?;

    let xml_content = fs::read_to_string(validator.model_description_path())?;

    // Parse XML and look for Float64 variables
    let mut reader = Reader::from_str(&xml_content);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut variable_count = 0;
    let mut in_model_variables = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.name().as_ref() == b"ModelVariables" {
                    in_model_variables = true;
                } else if in_model_variables && e.name().as_ref() == b"Float64" {
                    variable_count += 1;
                }
            }
            Ok(Event::Empty(ref e)) => {
                if in_model_variables && e.name().as_ref() == b"Float64" {
                    variable_count += 1;
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"ModelVariables" {
                    in_model_variables = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => anyhow::bail!("Error parsing XML: {:?}", e),
            _ => {}
        }
        buf.clear();
    }

    assert!(
        variable_count > 0,
        "No model variables found in modelDescription.xml"
    );

    println!("Found {} model variables in modelDescription.xml", variable_count);
    Ok(())
}

#[test]
fn test_fmu_cosimulation_attributes() -> Result<()> {
    let fmu_path = get_workspace_dir()?.join("BouncingBallLiaison.fmu");
    let validator = FmuValidator::new(fmu_path)?;
    validator.extract()?;

    let xml_content = fs::read_to_string(validator.model_description_path())?;

    // Parse XML and extract CoSimulation attributes
    let mut reader = Reader::from_str(&xml_content);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut cosim_attrs = HashMap::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) if e.name().as_ref() == b"CoSimulation" => {
                for attr in e.attributes() {
                    let attr = attr?;
                    let key = String::from_utf8(attr.key.as_ref().to_vec())?;
                    let value = String::from_utf8(attr.value.to_vec())?;
                    cosim_attrs.insert(key, value);
                }
                break;
            }
            Ok(Event::Eof) => break,
            Err(e) => anyhow::bail!("Error parsing XML: {:?}", e),
            _ => {}
        }
        buf.clear();
    }

    assert!(
        cosim_attrs.contains_key("modelIdentifier"),
        "CoSimulation element missing modelIdentifier"
    );

    println!("CoSimulation attributes:");
    for (key, value) in &cosim_attrs {
        println!("  - {}: {}", key, value);
    }

    Ok(())
}
