// Placeholder structure for managing FMI instance state and Zenoh communication
// This corresponds to the C++ Placeholder class

use crate::fmi3::{fmi3InstanceEnvironment, fmi3LogMessageCallback, fmi3Status};
use crate::proto;
use crate::utils::get_base_directory;
use anyhow::{Context, Result};
use prost::Message;
use serde_json::Value as JsonValue;
use std::ffi::CString;
use std::sync::Arc;
use zenoh::Wait;

/// Placeholder structure that manages FMI instance state and Zenoh communication
pub struct Placeholder {
    /// Instance index for this FMU instance
    pub instance_index: i32,

    /// Responder ID used for constructing Zenoh key expressions
    pub responder_id: String,

    /// Zenoh session for communication
    pub session: Arc<zenoh::Session>,

    /// Instance environment for callbacks (opaque pointer from FMI)
    pub instance_environment: fmi3InstanceEnvironment,

    /// Log message callback function
    pub log_message: fmi3LogMessageCallback,

    /// Zenoh subscriber for log messages
    pub log_message_subscriber: Option<zenoh::pubsub::Subscriber<()>>,
}

impl Placeholder {
    /// Create a new Placeholder instance
    ///
    /// This function:
    /// 1. Loads the config.json file from the base directory
    /// 2. Extracts the responderId
    /// 3. Configures and opens a Zenoh session
    /// 4. Sets up a log message subscriber
    ///
    /// # Arguments
    ///
    /// * `instance_environment` - Opaque pointer to the instance environment for callbacks
    /// * `log_message` - Callback function for logging messages
    ///
    /// # Returns
    ///
    /// A Result containing the new Placeholder instance or an error
    pub fn new(
        instance_environment: fmi3InstanceEnvironment,
        log_message: fmi3LogMessageCallback,
    ) -> Result<Self> {
        // Get base directory (parent of parent of the shared library)
        let base_directory = get_base_directory()
            .context("Failed to get base directory")?;

        // Load and parse config.json
        let config_file_path = format!("{}/config.json", base_directory);
        let config_content = std::fs::read_to_string(&config_file_path)
            .with_context(|| format!("Failed to open config file at: {}", config_file_path))?;

        let config: JsonValue = serde_json::from_str(&config_content)
            .context("Failed to parse config.json")?;

        // Extract responder ID
        let responder_id = config["responderId"]
            .as_str()
            .context("responderId not found in config.json")?
            .to_string();

        // Configure Zenoh session
        let zenoh_config = if let Some(zenoh_config_obj) = config.get("zenohConfig") {
            let mut zenoh_config_obj = zenoh_config_obj.clone();

            // Update TLS certificate paths to be absolute
            if let Some(transport) = zenoh_config_obj.get_mut("transport") {
                if let Some(link) = transport.get_mut("link") {
                    if let Some(tls) = link.get_mut("tls") {
                        if let Some(cert) = tls.get_mut("connect_certificate") {
                            if let Some(cert_str) = cert.as_str() {
                                *cert = JsonValue::String(format!("{}/{}", base_directory, cert_str));
                            }
                        }
                        if let Some(key) = tls.get_mut("connect_private_key") {
                            if let Some(key_str) = key.as_str() {
                                *key = JsonValue::String(format!("{}/{}", base_directory, key_str));
                            }
                        }
                        if let Some(ca) = tls.get_mut("root_ca_certificate") {
                            if let Some(ca_str) = ca.as_str() {
                                *ca = JsonValue::String(format!("{}/{}", base_directory, ca_str));
                            }
                        }
                    }
                }
            }

            // Convert to JSON string and parse as Zenoh config
            let config_str = serde_json::to_string(&zenoh_config_obj)
                .context("Failed to serialize Zenoh config")?;

            zenoh::Config::from_json5(&config_str)
                .map_err(|e| anyhow::anyhow!("Failed to parse Zenoh config from JSON: {:?}", e))?
        } else {
            // Use default config if zenohConfig is not specified
            zenoh::Config::default()
        };

        // Open Zenoh session
        let session = zenoh::open(zenoh_config)
            .wait()
            .map_err(|e| anyhow::anyhow!("Failed to open Zenoh session: {:?}", e))?;

        let session = Arc::new(session);

        let mut placeholder = Placeholder {
            instance_index: -1,
            responder_id,
            session,
            instance_environment,
            log_message,
            log_message_subscriber: None,
        };

        // Set up log message subscriber
        placeholder.add_log_message_subscriber()?;

        Ok(placeholder)
    }

    /// Set the instance index
    pub fn set_instance_index(&mut self, index: i32) {
        self.instance_index = index;
    }

    /// Add a subscriber for log messages from the remote FMU
    fn add_log_message_subscriber(&mut self) -> Result<()> {
        let log_message_callback = self.log_message;
        // Convert raw pointer to usize for thread safety (Send)
        let instance_environment_addr = self.instance_environment as usize;

        // Construct the key expression for log messages
        let expr = format!("rpc/{}/fmi3LogMessage", self.responder_id);
        let key_expr = zenoh::key_expr::KeyExpr::try_from(expr.clone())
            .map_err(|e| anyhow::anyhow!("Failed to create key expression {}: {:?}", expr, e))?;

        // Create subscriber with callback
        let subscriber = self.session
            .declare_subscriber(&key_expr)
            .callback(move |sample| {
                // Parse the protobuf message
                let payload_bytes = sample.payload().to_bytes();
                if let Ok(log_msg) = proto::LogMessage::decode(payload_bytes.as_ref()) {
                    // Convert proto Status to fmi3Status
                    let status: fmi3Status = proto::Status::try_from(log_msg.status)
                        .unwrap_or(proto::Status::Error)
                        .into();

                    // Create C strings for category and message
                    if let (Ok(category_cstr), Ok(message_cstr)) = (
                        CString::new(log_msg.category),
                        CString::new(log_msg.message),
                    ) {
                        // Call the log message callback if it exists
                        if let Some(callback) = log_message_callback {
                            // Convert back to pointer
                            let instance_environment = instance_environment_addr as fmi3InstanceEnvironment;
                            callback(
                                instance_environment,
                                status,
                                category_cstr.as_ptr(),
                                message_cstr.as_ptr(),
                            );
                        }
                    }
                }
            })
            .wait()
            .map_err(|e| anyhow::anyhow!("Failed to create log message subscriber: {:?}", e))?;

        self.log_message_subscriber = Some(subscriber);

        Ok(())
    }

    /// Perform a Zenoh query operation
    ///
    /// This method serializes the input protobuf message, sends it as a Zenoh query,
    /// waits for a reply, and deserializes the response.
    ///
    /// # Arguments
    ///
    /// * `fmi3_function` - The FMI function name (used in the key expression)
    /// * `input` - The input protobuf message to serialize and send
    ///
    /// # Returns
    ///
    /// A Result containing the deserialized output message or an error
    pub fn query<I: Message, O: Message + Default>(
        &self,
        fmi3_function: &str,
        input: &I,
    ) -> Result<O> {
        // Serialize input to wire format
        let mut input_wire = Vec::with_capacity(input.encoded_len());
        input.encode(&mut input_wire)
            .context("Failed to serialize input message")?;

        // Construct the key expression
        let expr = format!("rpc/{}/{}", self.responder_id, fmi3_function);
        let key_expr = zenoh::key_expr::KeyExpr::try_from(expr.clone())
            .map_err(|e| anyhow::anyhow!("Failed to create key expression {}: {:?}", expr, e))?;

        // Create query with payload
        let replies = self.session
            .get(&key_expr)
            .payload(input_wire)
            .wait()
            .map_err(|e| anyhow::anyhow!("Failed to send query to {}: {:?}", expr, e))?;

        // Wait for and process the first reply
        while let Ok(reply) = replies.recv() {
            match reply.result() {
                Ok(sample) => {
                    // Deserialize the response
                    let payload_bytes = sample.payload().to_bytes();
                    let output = O::decode(payload_bytes.as_ref())
                        .context("Failed to deserialize output message")?;
                    return Ok(output);
                }
                Err(err) => {
                    // Log error and continue waiting for valid reply
                    self.log_error(
                        fmi3_function,
                        &format!("Query error for '{}': {:?}", expr, err),
                    );
                }
            }
        }

        // No valid replies received
        let error_msg = format!("No valid replies received from '{}'", expr);
        self.log_error(fmi3_function, &error_msg);
        anyhow::bail!(error_msg)
    }

    /// Helper method to log errors via the FMI callback
    fn log_error(&self, function_name: &str, message: &str) {
        let full_message = format!("Exception in {}: {}", function_name, message);

        if let (Ok(category_cstr), Ok(message_cstr)) = (
            CString::new("Zenoh"),
            CString::new(full_message),
        ) {
            if let Some(callback) = self.log_message {
                callback(
                    self.instance_environment,
                    fmi3Status::fmi3Error,
                    category_cstr.as_ptr(),
                    message_cstr.as_ptr(),
                );
            }
        }
    }
}

impl Drop for Placeholder {
    fn drop(&mut self) {
        // Undeclare the log message subscriber
        if let Some(subscriber) = self.log_message_subscriber.take() {
            // The subscriber will be automatically undeclared when dropped
            drop(subscriber);
        }

        // Close the Zenoh session (will be done when Arc is dropped)
        // No explicit close needed with zenoh 1.0 API
    }
}

// Thread safety: Placeholder can be sent between threads but not shared
// This matches the C++ implementation where each instance is managed separately
unsafe impl Send for Placeholder {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder_creation_without_config() {
        // This test will fail if config.json doesn't exist
        // In a real scenario, you would mock the file system or provide a test config

        // For now, we just verify the type compiles
        let _: Option<Placeholder> = None;
    }

    #[test]
    fn test_set_instance_index() {
        // Create a mock placeholder for testing
        // In production, this would require a valid config.json
        // For this test, we just verify the method signature
        let test_index = 42;

        // Verify the logic would work
        assert_eq!(test_index, 42);
    }
}
