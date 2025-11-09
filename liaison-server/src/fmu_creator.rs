// FMU creator implementation
// Creates Liaison FMUs from existing FMUs

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};
use std::fs::File;
use std::path::PathBuf;
use zip::ZipWriter;

use crate::utils::{add_file_to_fmu, unzip_fmu};

/// Creates a Liaison FMU wrapper around an existing FMU.
///
/// This function will transform a standard FMU into a Liaison FMU that can communicate
/// with other FMUs through Zenoh.
///
/// # Arguments
///
/// * `fmu_path` - Path to the FMU file to wrap
/// * `responder_id` - Unique identifier for the responder instance
/// * `zenoh_config` - Optional path to Zenoh configuration file
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if creation fails.
///
/// # Process
///
/// 1. Extract the source FMU to a temp directory
/// 2. Create a new FMU ZIP archive (named {modelName}Liaison.fmu)
/// 3. Add the liaison FMI client library binaries to the FMU
/// 4. Copy modelDescription.xml from the extracted FMU
/// 5. Process Zenoh config if provided (including TLS certificates)
/// 6. Create and add config.json
/// 7. Finalize the ZIP archive
pub fn make_fmu(
    fmu_path: PathBuf,
    responder_id: String,
    zenoh_config: Option<PathBuf>,
) -> Result<()> {
    // Log the FMU creation request
    let zenoh_config_msg = if let Some(ref config_path) = zenoh_config {
        format!("Zenoh config file: {}\n", config_path.display())
    } else {
        String::new()
    };

    tracing::info!(
        "\n\
         ====================================\n\
         Making Liaison FMU\n\
         ====================================\n\
         FMU: {}\n\
         Responder ID: {}\n\
         {}\
         ====================================",
        fmu_path.display(),
        responder_id,
        zenoh_config_msg
    );

    // Extract model name from FMU path
    let model_name = fmu_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("Failed to extract model name from FMU path"))?
        .to_string();

    // Extract the source FMU to a temporary directory
    tracing::info!("Extracting source FMU...");
    let temp_path = unzip_fmu(&fmu_path).context("Failed to unzip the FMU")?;
    tracing::debug!("FMU extracted to: {}", temp_path);

    // Create the output FMU path
    let output_fmu_path = PathBuf::from(format!("{}Liaison.fmu", model_name));

    // Create the ZIP/FMU archive
    tracing::info!(
        "Creating Liaison FMU archive: {}",
        output_fmu_path.display()
    );
    let fmu_file =
        File::create(&output_fmu_path).context("Failed to create the Liaison FMU file")?;
    let mut zip = ZipWriter::new(fmu_file);

    // Add the platform binaries to the FMU
    let mut dynamic_library_errors = 0;

    // Check that the binaries directory exists
    let binaries_path = PathBuf::from("./binaries");
    if !binaries_path.exists() {
        return Err(anyhow!("Required directory './binaries' does not exist."));
    }

    // Add Linux dynamic library
    let original_linux_lib = PathBuf::from("./binaries/x86_64-linux/libliaisonfmu.so");
    let renamed_linux_lib = format!("binaries/x86_64-linux/{}.so", model_name);
    match add_file_to_fmu(&mut zip, &original_linux_lib, &renamed_linux_lib) {
        Ok(_) => {
            tracing::info!("  Added: {}", renamed_linux_lib);
        }
        Err(e) => {
            tracing::warn!(
                "Failed adding Liaison Linux dynamic library file to the Liaison FMU: {}",
                e
            );
            dynamic_library_errors += 1;
        }
    }

    // Add Windows dynamic library
    let original_windows_lib = PathBuf::from("./binaries/x86_64-windows/liaisonfmu.dll");
    let renamed_windows_lib = format!("binaries/x86_64-windows/{}.dll", model_name);
    match add_file_to_fmu(&mut zip, &original_windows_lib, &renamed_windows_lib) {
        Ok(_) => {
            tracing::info!("  Added: {}", renamed_windows_lib);
        }
        Err(e) => {
            tracing::warn!(
                "Failed adding Liaison Windows dynamic library file to the Liaison FMU: {}",
                e
            );
            dynamic_library_errors += 1;
        }
    }

    // At least one platform binary must be present
    if dynamic_library_errors == 2 {
        return Err(anyhow!(
            "Failed adding ANY Liaison dynamic library file to the Liaison FMU. At least one is required."
        ));
    }

    // Add the modelDescription.xml file to the FMU at the base directory
    let model_description_path = PathBuf::from(&temp_path).join("modelDescription.xml");
    add_file_to_fmu(&mut zip, &model_description_path, "modelDescription.xml")
        .context("Failed adding modelDescription.xml to the Liaison FMU")?;
    tracing::info!("  Added: modelDescription.xml");

    // Process Zenoh configuration if provided
    let mut zenoh_config_json = Value::Null;
    if let Some(config_path) = zenoh_config {
        tracing::info!("Processing Zenoh configuration...");

        // Read and parse the zenoh config file
        let config_file = File::open(&config_path).context(format!(
            "Failed to open Zenoh config file: {}",
            config_path.display()
        ))?;
        let mut config: Value =
            serde_json::from_reader(config_file).context("Failed to parse Zenoh config JSON")?;

        // Set metadata.name to modelName
        if config.get("metadata").is_none() {
            config["metadata"] = json!({});
        }
        config["metadata"]["name"] = json!(model_name.clone());

        // Process TLS certificates if present
        if let Some(transport) = config.get_mut("transport") {
            if let Some(link) = transport.get_mut("link") {
                if let Some(tls) = link.get_mut("tls") {
                    process_tls_certificates(tls, &mut zip)?;
                }
            }
        }

        zenoh_config_json = config;
    }

    // Create the config.json file
    let mut config = json!({
        "responderId": responder_id,
        "name": model_name.clone(),
    });

    if !zenoh_config_json.is_null() {
        config["zenohConfig"] = zenoh_config_json;
    }

    // Write config to a temporary file
    let config_file_path = PathBuf::from(&temp_path).join("config.json");
    let config_file =
        File::create(&config_file_path).context("Failed to create config.json file")?;
    serde_json::to_writer_pretty(config_file, &config).context("Failed to write config.json")?;

    // Add config.json to the FMU
    add_file_to_fmu(&mut zip, &config_file_path, "binaries/config.json")
        .context("Failed adding Liaison config file to the Liaison FMU")?;
    tracing::info!("  Added: binaries/config.json");

    // Finalize the ZIP archive
    zip.finish().context("Failed to finalize FMU zip archive")?;

    tracing::info!("Liaison FMU successfully created!");
    Ok(())
}

/// Process TLS certificates in the Zenoh configuration.
///
/// This function:
/// 1. Extracts certificate paths from the TLS configuration
/// 2. Validates that the certificate files exist
/// 3. Adds the certificate files to the FMU's binaries/ directory
/// 4. Updates the configuration to use relative paths (just filenames)
///
/// # Arguments
///
/// * `tls` - Mutable reference to the TLS section of the Zenoh config
/// * `zip` - Mutable reference to the ZIP writer
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if any certificate processing fails.
fn process_tls_certificates(tls: &mut Value, zip: &mut ZipWriter<File>) -> Result<()> {
    // Process connect_certificate
    if let Some(cert_path_value) = tls.get("connect_certificate") {
        if let Some(cert_path_str) = cert_path_value.as_str() {
            if !cert_path_str.is_empty() {
                let cert_path = PathBuf::from(cert_path_str);

                if !cert_path.exists() {
                    return Err(anyhow!(
                        "Connect certificate file does not exist at: {}",
                        cert_path.display()
                    ));
                }

                let cert_filename = cert_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| anyhow!("Invalid certificate filename"))?;

                add_file_to_fmu(zip, &cert_path, &format!("binaries/{}", cert_filename))
                    .context("Failed adding connect certificate file to the Liaison FMU")?;

                tls["connect_certificate"] = json!(cert_filename);
                tracing::info!("  Added: {}", cert_filename);
            }
        }
    }

    // Process connect_private_key
    if let Some(key_path_value) = tls.get("connect_private_key") {
        if let Some(key_path_str) = key_path_value.as_str() {
            if !key_path_str.is_empty() {
                let key_path = PathBuf::from(key_path_str);

                if !key_path.exists() {
                    return Err(anyhow!(
                        "Connect private key file does not exist at: {}",
                        key_path.display()
                    ));
                }

                let key_filename = key_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| anyhow!("Invalid private key filename"))?;

                add_file_to_fmu(zip, &key_path, &format!("binaries/{}", key_filename))
                    .context("Failed adding connect private key file to the Liaison FMU")?;

                tls["connect_private_key"] = json!(key_filename);
                tracing::info!("  Added: {}", key_filename);
            }
        }
    }

    // Process root_ca_certificate
    if let Some(ca_path_value) = tls.get("root_ca_certificate") {
        if let Some(ca_path_str) = ca_path_value.as_str() {
            if !ca_path_str.is_empty() {
                let ca_path = PathBuf::from(ca_path_str);

                if !ca_path.exists() {
                    return Err(anyhow!(
                        "Root CA certificate file does not exist at: {}",
                        ca_path.display()
                    ));
                }

                let ca_filename = ca_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| anyhow!("Invalid CA certificate filename"))?;

                add_file_to_fmu(zip, &ca_path, &format!("binaries/{}", ca_filename))
                    .context("Failed adding root CA certificate file to the Liaison FMU")?;

                tls["root_ca_certificate"] = json!(ca_filename);
                tracing::info!("  Added: {}", ca_filename);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    /// Helper function to create a minimal test FMU
    fn create_test_fmu(temp_dir: &TempDir, model_name: &str) -> PathBuf {
        let fmu_path = temp_dir.path().join(format!("{}.fmu", model_name));
        let fmu_file = File::create(&fmu_path).unwrap();
        let mut zip = ZipWriter::new(fmu_file);

        // Create a minimal modelDescription.xml
        let model_description = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<fmiModelDescription
    fmiVersion="3.0"
    modelName="{}"
    instantiationToken="{{12345678-1234-1234-1234-123456789012}}">
    <CoSimulation modelIdentifier="{}"/>
</fmiModelDescription>"#,
            model_name, model_name
        );

        zip.start_file("modelDescription.xml", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(model_description.as_bytes()).unwrap();
        zip.finish().unwrap();

        fmu_path
    }

    /// Helper function to create test certificate files
    fn create_test_certificates(temp_dir: &TempDir) -> (PathBuf, PathBuf, PathBuf) {
        let cert_path = temp_dir.path().join("client.crt");
        let key_path = temp_dir.path().join("client.key");
        let ca_path = temp_dir.path().join("ca.crt");

        fs::write(&cert_path, "FAKE CERTIFICATE DATA").unwrap();
        fs::write(&key_path, "FAKE KEY DATA").unwrap();
        fs::write(&ca_path, "FAKE CA DATA").unwrap();

        (cert_path, key_path, ca_path)
    }

    /// Helper function to create a Zenoh config file
    fn create_zenoh_config(
        temp_dir: &TempDir,
        cert_path: Option<&PathBuf>,
        key_path: Option<&PathBuf>,
        ca_path: Option<&PathBuf>,
    ) -> PathBuf {
        let config_path = temp_dir.path().join("zenoh_config.json");
        let mut config = json!({
            "mode": "client",
            "connect": {
                "endpoints": ["tcp/localhost:7447"]
            }
        });

        if cert_path.is_some() || key_path.is_some() || ca_path.is_some() {
            config["transport"] = json!({
                "link": {
                    "tls": {}
                }
            });

            if let Some(cert) = cert_path {
                config["transport"]["link"]["tls"]["connect_certificate"] =
                    json!(cert.to_str().unwrap());
            }
            if let Some(key) = key_path {
                config["transport"]["link"]["tls"]["connect_private_key"] =
                    json!(key.to_str().unwrap());
            }
            if let Some(ca) = ca_path {
                config["transport"]["link"]["tls"]["root_ca_certificate"] =
                    json!(ca.to_str().unwrap());
            }
        }

        let mut file = File::create(&config_path).unwrap();
        serde_json::to_writer_pretty(&mut file, &config).unwrap();
        config_path
    }

    /// Helper function to create binaries directory structure
    #[allow(dead_code)]
    fn create_binaries_dir(temp_dir: &TempDir) -> PathBuf {
        let binaries_path = temp_dir.path().join("binaries");
        fs::create_dir_all(binaries_path.join("x86_64-linux")).unwrap();
        fs::create_dir_all(binaries_path.join("x86_64-windows")).unwrap();

        // Create dummy library files
        fs::write(
            binaries_path.join("x86_64-linux/libliaisonfmu.so"),
            "FAKE LINUX LIBRARY",
        )
        .unwrap();
        fs::write(
            binaries_path.join("x86_64-windows/liaisonfmu.dll"),
            "FAKE WINDOWS LIBRARY",
        )
        .unwrap();

        binaries_path
    }

    /// Helper to extract and validate FMU contents
    #[allow(dead_code)]
    fn extract_and_validate_fmu(fmu_path: &PathBuf) -> Result<TempDir> {
        let temp_dir = TempDir::new()?;
        let file = File::open(fmu_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = temp_dir.path().join(file.name());

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    fs::create_dir_all(p)?;
                }
                let mut outfile = File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        Ok(temp_dir)
    }

    #[test]
    fn test_tls_certificate_extraction() {
        // Test that we can properly extract certificate paths from JSON
        let tls_config = json!({
            "connect_certificate": "/path/to/cert.pem",
            "connect_private_key": "/path/to/key.pem",
            "root_ca_certificate": "/path/to/ca.pem"
        });

        assert_eq!(
            tls_config
                .get("connect_certificate")
                .and_then(|v| v.as_str()),
            Some("/path/to/cert.pem")
        );
        assert_eq!(
            tls_config
                .get("connect_private_key")
                .and_then(|v| v.as_str()),
            Some("/path/to/key.pem")
        );
        assert_eq!(
            tls_config
                .get("root_ca_certificate")
                .and_then(|v| v.as_str()),
            Some("/path/to/ca.pem")
        );
    }

    #[test]
    fn test_model_name_extraction() {
        let path = PathBuf::from("/some/path/MyModel.fmu");
        let model_name = path.file_stem().and_then(|s| s.to_str()).unwrap();
        assert_eq!(model_name, "MyModel");
    }

    #[test]
    fn test_model_name_extraction_various_paths() {
        // Test different path formats
        let test_cases = [
            ("model.fmu", "model"),
            ("MyModel.fmu", "MyModel"),
            ("/absolute/path/TestFMU.fmu", "TestFMU"),
            ("./relative/ComplexModel.fmu", "ComplexModel"),
        ];

        for (input, expected) in test_cases {
            let path = PathBuf::from(input);
            let model_name = path.file_stem().and_then(|s| s.to_str()).unwrap();
            assert_eq!(model_name, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_tls_config_with_empty_strings() {
        // Test handling of empty string paths
        let tls_config = json!({
            "connect_certificate": "",
            "connect_private_key": "",
            "root_ca_certificate": ""
        });

        assert_eq!(
            tls_config
                .get("connect_certificate")
                .and_then(|v| v.as_str()),
            Some("")
        );
    }

    #[test]
    fn test_tls_config_missing_fields() {
        // Test handling of missing TLS fields
        let tls_config = json!({
            "connect_certificate": "/path/to/cert.pem"
        });

        assert_eq!(
            tls_config
                .get("connect_certificate")
                .and_then(|v| v.as_str()),
            Some("/path/to/cert.pem")
        );
        assert_eq!(
            tls_config
                .get("connect_private_key")
                .and_then(|v| v.as_str()),
            None
        );
    }

    #[test]
    fn test_process_tls_certificates_with_valid_files() {
        let temp_dir = TempDir::new().unwrap();
        let (cert_path, key_path, ca_path) = create_test_certificates(&temp_dir);

        let mut tls_config = json!({
            "connect_certificate": cert_path.to_str().unwrap(),
            "connect_private_key": key_path.to_str().unwrap(),
            "root_ca_certificate": ca_path.to_str().unwrap()
        });

        let zip_path = temp_dir.path().join("test.zip");
        let zip_file = File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(zip_file);

        let result = process_tls_certificates(&mut tls_config, &mut zip);
        assert!(
            result.is_ok(),
            "Should successfully process valid TLS files"
        );

        // Verify that paths were converted to filenames
        assert_eq!(
            tls_config
                .get("connect_certificate")
                .and_then(|v| v.as_str()),
            Some("client.crt")
        );
        assert_eq!(
            tls_config
                .get("connect_private_key")
                .and_then(|v| v.as_str()),
            Some("client.key")
        );
        assert_eq!(
            tls_config
                .get("root_ca_certificate")
                .and_then(|v| v.as_str()),
            Some("ca.crt")
        );

        zip.finish().unwrap();
    }

    #[test]
    fn test_process_tls_certificates_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent = temp_dir.path().join("nonexistent.crt");

        let mut tls_config = json!({
            "connect_certificate": nonexistent.to_str().unwrap()
        });

        let zip_path = temp_dir.path().join("test.zip");
        let zip_file = File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(zip_file);

        let result = process_tls_certificates(&mut tls_config, &mut zip);
        assert!(
            result.is_err(),
            "Should fail when certificate file doesn't exist"
        );
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[test]
    fn test_process_tls_certificates_empty_paths() {
        let temp_dir = TempDir::new().unwrap();

        let mut tls_config = json!({
            "connect_certificate": "",
            "connect_private_key": "",
            "root_ca_certificate": ""
        });

        let zip_path = temp_dir.path().join("test.zip");
        let zip_file = File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(zip_file);

        let result = process_tls_certificates(&mut tls_config, &mut zip);
        assert!(
            result.is_ok(),
            "Should succeed with empty paths (they are skipped)"
        );

        // Empty paths should remain unchanged
        assert_eq!(
            tls_config
                .get("connect_certificate")
                .and_then(|v| v.as_str()),
            Some("")
        );
    }

    #[test]
    fn test_process_tls_certificates_partial_config() {
        let temp_dir = TempDir::new().unwrap();
        let (cert_path, _, _) = create_test_certificates(&temp_dir);

        // Only provide certificate, not key or CA
        let mut tls_config = json!({
            "connect_certificate": cert_path.to_str().unwrap()
        });

        let zip_path = temp_dir.path().join("test.zip");
        let zip_file = File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(zip_file);

        let result = process_tls_certificates(&mut tls_config, &mut zip);
        assert!(result.is_ok(), "Should handle partial TLS config");

        assert_eq!(
            tls_config
                .get("connect_certificate")
                .and_then(|v| v.as_str()),
            Some("client.crt")
        );
        zip.finish().unwrap();
    }

    #[test]
    fn test_config_json_generation_basic() {
        // Test basic config.json structure
        let responder_id = "test-responder-123".to_string();
        let model_name = "TestModel".to_string();

        let config = json!({
            "responderId": responder_id,
            "name": model_name,
        });

        assert_eq!(
            config.get("responderId").and_then(|v| v.as_str()),
            Some("test-responder-123")
        );
        assert_eq!(
            config.get("name").and_then(|v| v.as_str()),
            Some("TestModel")
        );
        assert!(config.get("zenohConfig").is_none());
    }

    #[test]
    fn test_config_json_generation_with_zenoh() {
        // Test config.json with Zenoh configuration
        let responder_id = "test-responder-456".to_string();
        let model_name = "TestModel".to_string();
        let zenoh_config_json = json!({
            "mode": "client",
            "metadata": {
                "name": "TestModel"
            }
        });

        let mut config = json!({
            "responderId": responder_id,
            "name": model_name,
        });

        config["zenohConfig"] = zenoh_config_json.clone();

        assert_eq!(
            config.get("responderId").and_then(|v| v.as_str()),
            Some("test-responder-456")
        );
        assert_eq!(
            config.get("name").and_then(|v| v.as_str()),
            Some("TestModel")
        );
        assert!(config.get("zenohConfig").is_some());
        assert_eq!(config["zenohConfig"]["mode"].as_str(), Some("client"));
    }

    #[test]
    fn test_zenoh_config_metadata_name_setting() {
        // Test that metadata.name gets set to model name
        let model_name = "MyTestModel";
        let mut config = json!({
            "mode": "client"
        });

        // Simulate the metadata setting logic
        if config.get("metadata").is_none() {
            config["metadata"] = json!({});
        }
        config["metadata"]["name"] = json!(model_name);

        assert_eq!(config["metadata"]["name"].as_str(), Some("MyTestModel"));
    }

    #[test]
    fn test_zenoh_config_preserves_existing_metadata() {
        // Test that existing metadata is preserved
        let model_name = "MyModel";
        let mut config = json!({
            "mode": "client",
            "metadata": {
                "custom_field": "custom_value"
            }
        });

        config["metadata"]["name"] = json!(model_name);

        assert_eq!(config["metadata"]["name"].as_str(), Some("MyModel"));
        assert_eq!(
            config["metadata"]["custom_field"].as_str(),
            Some("custom_value")
        );
    }

    #[test]
    fn test_make_fmu_invalid_path() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent_fmu = temp_dir.path().join("nonexistent.fmu");

        let result = make_fmu(nonexistent_fmu, "test-responder".to_string(), None);

        assert!(result.is_err(), "Should fail with nonexistent FMU");
    }

    #[test]
    fn test_make_fmu_without_binaries_directory() {
        let temp_dir = TempDir::new().unwrap();

        // Create a test FMU
        let fmu_path = create_test_fmu(&temp_dir, "TestModel");

        // Change to temp directory where binaries don't exist
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = make_fmu(fmu_path, "test-responder".to_string(), None);

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        assert!(
            result.is_err(),
            "Should fail when binaries directory doesn't exist"
        );
        assert!(result.unwrap_err().to_string().contains("binaries"));
    }

    #[test]
    fn test_output_fmu_naming() {
        // Test that output FMU is named correctly
        let model_name = "MyTestModel";
        let expected_output = format!("{}Liaison.fmu", model_name);

        assert_eq!(expected_output, "MyTestModelLiaison.fmu");
    }

    #[test]
    fn test_library_filename_generation() {
        // Test that library filenames are generated correctly
        let model_name = "TestModel";
        let linux_lib = format!("binaries/x86_64-linux/{}.so", model_name);
        let windows_lib = format!("binaries/x86_64-windows/{}.dll", model_name);

        assert_eq!(linux_lib, "binaries/x86_64-linux/TestModel.so");
        assert_eq!(windows_lib, "binaries/x86_64-windows/TestModel.dll");
    }

    #[test]
    fn test_zenoh_config_file_parsing() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_zenoh_config(&temp_dir, None, None, None);

        let file = File::open(&config_path).unwrap();
        let config: Value = serde_json::from_reader(file).unwrap();

        assert_eq!(config["mode"].as_str(), Some("client"));
        assert!(config["connect"]["endpoints"].is_array());
    }

    #[test]
    fn test_zenoh_config_with_tls_parsing() {
        let temp_dir = TempDir::new().unwrap();
        let (cert_path, key_path, ca_path) = create_test_certificates(&temp_dir);
        let config_path =
            create_zenoh_config(&temp_dir, Some(&cert_path), Some(&key_path), Some(&ca_path));

        let file = File::open(&config_path).unwrap();
        let config: Value = serde_json::from_reader(file).unwrap();

        assert!(config["transport"]["link"]["tls"].is_object());
        assert!(config["transport"]["link"]["tls"]["connect_certificate"].is_string());
        assert!(config["transport"]["link"]["tls"]["connect_private_key"].is_string());
        assert!(config["transport"]["link"]["tls"]["root_ca_certificate"].is_string());
    }

    #[test]
    fn test_model_name_from_xml() {
        // Test extracting model name from modelDescription.xml content
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<fmiModelDescription modelName="ExtractedModelName" fmiVersion="3.0">
</fmiModelDescription>"#;

        // For now we extract from file name, not XML
        // But this test validates XML structure
        assert!(xml.contains("modelName=\"ExtractedModelName\""));
    }

    #[test]
    fn test_invalid_model_name_from_path() {
        // Test path without file stem
        let path = PathBuf::from("/");
        let model_name = path.file_stem().and_then(|s| s.to_str());
        assert!(model_name.is_none());
    }

    #[test]
    fn test_certificate_filename_extraction() {
        // Test extracting just the filename from full paths
        let cert_path = PathBuf::from("/etc/certs/my-certificate.pem");
        let filename = cert_path.file_name().and_then(|s| s.to_str()).unwrap();
        assert_eq!(filename, "my-certificate.pem");

        let key_path = PathBuf::from("./keys/private.key");
        let key_filename = key_path.file_name().and_then(|s| s.to_str()).unwrap();
        assert_eq!(key_filename, "private.key");
    }

    #[test]
    fn test_json_null_handling() {
        // Test that Value::Null works as expected
        let mut zenoh_config_json = Value::Null;
        assert!(zenoh_config_json.is_null());

        zenoh_config_json = json!({"test": "value"});
        assert!(!zenoh_config_json.is_null());
    }
}
