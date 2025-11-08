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
    zip.finish()
        .context("Failed to finalize FMU zip archive")?;

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
    }

    #[test]
    fn test_model_name_extraction() {
        let path = PathBuf::from("/some/path/MyModel.fmu");
        let model_name = path.file_stem().and_then(|s| s.to_str()).unwrap();
        assert_eq!(model_name, "MyModel");
    }
}
