//! FMU Server Implementation
//!
//! This module implements the main server functionality for serving FMU instances
//! over the Zenoh network. It handles:
//! - FMU library loading and initialization
//! - Zenoh session setup and configuration
//! - Queryable declarations for all FMI 3.0 functions
//! - Server lifecycle management
//!
//! # Implementation Notes
//!
//! Based on C++ implementation in src/liaison.cpp (lines 755-915)

use crate::fmu_loader::{construct_library_path, FmuLibrary};
use crate::instance_manager::InstanceManager;
use crate::queryable_handlers;
use crate::utils::unzip_fmu;
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info};
use zenoh::Wait;

/// Start the FMU server
///
/// This function implements the complete server startup and runtime logic:
/// 1. Extract and load the FMU library
/// 2. Initialize Zenoh session
/// 3. Declare queryables for all FMI functions
/// 4. Run the server loop until shutdown
/// 5. Clean up resources
///
/// # Arguments
///
/// * `fmu_path` - Path to the FMU file (.fmu archive)
/// * `responder_id` - Unique identifier for this server instance
/// * `zenoh_config` - Optional path to Zenoh configuration file
/// * `python_env` - Optional path to Python environment (currently unused)
/// * `debug` - Enable debug logging
///
/// # Returns
///
/// A Result indicating success or failure
///
/// # Implementation Notes
///
/// Based on C++ implementation in src/liaison.cpp (lines 755-915)
pub fn start_server(
    fmu_path: PathBuf,
    responder_id: String,
    zenoh_config: Option<PathBuf>,
    _python_env: Option<PathBuf>,
    debug: bool,
) -> Result<()> {
    // Log startup banner
    let zenoh_config_str = zenoh_config
        .as_ref()
        .map(|p| format!("Zenoh config file: {}\n", p.display()))
        .unwrap_or_default();
    let debug_str = if debug { "DEBUG ENABLED\n" } else { "" };

    info!(
        "\n\
        ====================================\n\
        Serving FMU\n\
        ====================================\n\
        FMU: {}\n\
        Responder ID: {}\n\
        {}\
        {}\
        ====================================",
        fmu_path.display(),
        responder_id,
        zenoh_config_str,
        debug_str
    );

    //=========================================================================
    // 1. Extract FMU Information and Load Library
    //=========================================================================

    // Get model name from FMU file path
    let model_name = fmu_path
        .file_stem()
        .and_then(|s| s.to_str())
        .context("Invalid FMU file name")?
        .to_string();

    // Unzip FMU to temporary directory
    info!("Extracting FMU...");
    let temp_path = unzip_fmu(&fmu_path).context("Failed to extract FMU")?;
    debug!("FMU extracted to: {}", temp_path);

    // Construct library path
    let lib_path = construct_library_path(&temp_path, &model_name);
    debug!("FMU library path: {}", lib_path);

    // Set resource path
    let resource_path = format!("{}/resources", temp_path);
    debug!("Resource path: {}", resource_path);

    // Load FMU library
    info!("Loading FMU library...");
    let fmu = Arc::new(FmuLibrary::new(&lib_path).context("Failed to load FMU library")?);
    info!("FMU library loaded successfully");

    //=========================================================================
    // 2. Initialize Zenoh Session
    //=========================================================================

    info!("Starting Zenoh session...");
    let zenoh_config = if let Some(config_path) = zenoh_config {
        // Load config from file
        let config_str = std::fs::read_to_string(&config_path).with_context(|| {
            format!(
                "Failed to read Zenoh config file: {}",
                config_path.display()
            )
        })?;

        let config: zenoh::Config = serde_json::from_str(&config_str)
            .with_context(|| format!("Failed to parse Zenoh config: {}", config_path.display()))?;
        config
    } else {
        // Use default config
        zenoh::Config::default()
    };

    let session = Arc::new(
        zenoh::open(zenoh_config)
            .wait()
            .map_err(|e| anyhow::anyhow!("Failed to open Zenoh session: {:?}", e))?,
    );
    info!("Zenoh session opened");

    //=========================================================================
    // 3. Create LogMessage Publisher
    //=========================================================================

    let log_message_key = format!("rpc/{}/fmi3LogMessage", responder_id);
    let log_publisher = Arc::new(
        session
            .declare_publisher(log_message_key.clone())
            .wait()
            .map_err(|e| anyhow::anyhow!("Failed to declare log message publisher: {:?}", e))?,
    );
    debug!("Log message publisher declared: {}", log_message_key);

    //=========================================================================
    // 4. Create Instance Manager
    //=========================================================================

    let instance_manager = Arc::new(InstanceManager::new());
    debug!("Instance manager created");

    //=========================================================================
    // 5. Declare Queryables for All FMI Functions
    //=========================================================================

    info!("Declaring queryables...");

    // Clone Arc references for closures
    let fmu_clone = Arc::clone(&fmu);
    let instance_manager_clone = Arc::clone(&instance_manager);
    let resource_path_clone = resource_path.clone();
    let log_publisher_clone = Arc::clone(&log_publisher);

    // Helper macro to declare queryables (4 parameter handlers - most handlers)
    macro_rules! declare_queryable {
        ($func_name:expr, $handler:expr) => {{
            let key = format!("rpc/{}/{}", responder_id, $func_name);
            let fmu = Arc::clone(&fmu_clone);
            let instance_manager = Arc::clone(&instance_manager_clone);
            let resource_path = resource_path_clone.clone();

            let queryable = session
                .declare_queryable(&key)
                .callback(move |query| {
                    if let Err(e) =
                        $handler(query, &fmu, Arc::clone(&instance_manager), &resource_path)
                    {
                        error!("Error handling {}: {:?}", $func_name, e);
                    }
                })
                .wait()
                .map_err(|e| {
                    anyhow::anyhow!("Failed to declare queryable for {}: {:?}", $func_name, e)
                })?;

            debug!("Declared queryable: {}", key);
            queryable
        }};
    }

    // Helper macro for instantiation handlers (5 parameters - includes log_publisher)
    macro_rules! declare_instantiation_queryable {
        ($func_name:expr, $handler:expr) => {{
            let key = format!("rpc/{}/{}", responder_id, $func_name);
            let fmu = Arc::clone(&fmu_clone);
            let instance_manager = Arc::clone(&instance_manager_clone);
            let resource_path = resource_path_clone.clone();
            let log_pub = Arc::clone(&log_publisher_clone);

            let queryable = session
                .declare_queryable(&key)
                .callback(move |query| {
                    if let Err(e) = $handler(
                        query,
                        &fmu,
                        Arc::clone(&instance_manager),
                        &resource_path,
                        Arc::clone(&log_pub),
                    ) {
                        error!("Error handling {}: {:?}", $func_name, e);
                    }
                })
                .wait()
                .map_err(|e| {
                    anyhow::anyhow!("Failed to declare queryable for {}: {:?}", $func_name, e)
                })?;

            debug!("Declared queryable: {}", key);
            queryable
        }};
    }

    // Declare all queryables (based on liaison.cpp lines 832-870)
    let _queryable_set_debug_logging = declare_queryable!(
        "fmi3SetDebugLogging",
        queryable_handlers::handle_set_debug_logging
    );

    let _queryable_instantiate_co_simulation = declare_instantiation_queryable!(
        "fmi3InstantiateCoSimulation",
        queryable_handlers::handle_instantiate_co_simulation
    );

    let _queryable_instantiate_model_exchange = declare_instantiation_queryable!(
        "fmi3InstantiateModelExchange",
        queryable_handlers::handle_instantiate_model_exchange
    );

    let _queryable_instantiate_scheduled_execution = declare_instantiation_queryable!(
        "fmi3InstantiateScheduledExecution",
        queryable_handlers::handle_instantiate_scheduled_execution
    );

    let _queryable_enter_initialization_mode = declare_queryable!(
        "fmi3EnterInitializationMode",
        queryable_handlers::handle_enter_initialization_mode
    );

    let _queryable_exit_initialization_mode = declare_queryable!(
        "fmi3ExitInitializationMode",
        queryable_handlers::handle_exit_initialization_mode
    );

    let _queryable_enter_event_mode = declare_queryable!(
        "fmi3EnterEventMode",
        queryable_handlers::handle_enter_event_mode
    );

    let _queryable_free_instance =
        declare_queryable!("fmi3FreeInstance", queryable_handlers::handle_free_instance);

    let _queryable_do_step = declare_queryable!("fmi3DoStep", queryable_handlers::handle_do_step);

    // Get/Set Float32
    let _queryable_set_float32 =
        declare_queryable!("fmi3SetFloat32", queryable_handlers::handle_set_float32);

    let _queryable_get_float32 =
        declare_queryable!("fmi3GetFloat32", queryable_handlers::handle_get_float32);

    // Get/Set Float64
    let _queryable_set_float64 =
        declare_queryable!("fmi3SetFloat64", queryable_handlers::handle_set_float64);

    let _queryable_get_float64 =
        declare_queryable!("fmi3GetFloat64", queryable_handlers::handle_get_float64);

    // Get/Set Int8
    let _queryable_set_int8 =
        declare_queryable!("fmi3SetInt8", queryable_handlers::handle_set_int8);

    let _queryable_get_int8 =
        declare_queryable!("fmi3GetInt8", queryable_handlers::handle_get_int8);

    // Get/Set UInt8
    let _queryable_set_uint8 =
        declare_queryable!("fmi3SetUInt8", queryable_handlers::handle_set_uint8);

    let _queryable_get_uint8 =
        declare_queryable!("fmi3GetUInt8", queryable_handlers::handle_get_uint8);

    // Get/Set Int16
    let _queryable_set_int16 =
        declare_queryable!("fmi3SetInt16", queryable_handlers::handle_set_int16);

    let _queryable_get_int16 =
        declare_queryable!("fmi3GetInt16", queryable_handlers::handle_get_int16);

    // Get/Set UInt16
    let _queryable_set_uint16 =
        declare_queryable!("fmi3SetUInt16", queryable_handlers::handle_set_uint16);

    let _queryable_get_uint16 =
        declare_queryable!("fmi3GetUInt16", queryable_handlers::handle_get_uint16);

    // Get/Set Int32
    let _queryable_set_int32 =
        declare_queryable!("fmi3SetInt32", queryable_handlers::handle_set_int32);

    let _queryable_get_int32 =
        declare_queryable!("fmi3GetInt32", queryable_handlers::handle_get_int32);

    // Get/Set UInt32
    let _queryable_set_uint32 =
        declare_queryable!("fmi3SetUInt32", queryable_handlers::handle_set_uint32);

    let _queryable_get_uint32 =
        declare_queryable!("fmi3GetUInt32", queryable_handlers::handle_get_uint32);

    // Get/Set Int64
    let _queryable_set_int64 =
        declare_queryable!("fmi3SetInt64", queryable_handlers::handle_set_int64);

    let _queryable_get_int64 =
        declare_queryable!("fmi3GetInt64", queryable_handlers::handle_get_int64);

    // Get/Set UInt64
    let _queryable_set_uint64 =
        declare_queryable!("fmi3SetUInt64", queryable_handlers::handle_set_uint64);

    let _queryable_get_uint64 =
        declare_queryable!("fmi3GetUInt64", queryable_handlers::handle_get_uint64);

    // Get/Set Boolean
    let _queryable_set_boolean =
        declare_queryable!("fmi3SetBoolean", queryable_handlers::handle_set_boolean);

    let _queryable_get_boolean =
        declare_queryable!("fmi3GetBoolean", queryable_handlers::handle_get_boolean);

    // Get/Set String
    let _queryable_set_string =
        declare_queryable!("fmi3SetString", queryable_handlers::handle_set_string);

    let _queryable_get_string =
        declare_queryable!("fmi3GetString", queryable_handlers::handle_get_string);

    // Get/Set Clock
    let _queryable_set_clock =
        declare_queryable!("fmi3SetClock", queryable_handlers::handle_set_clock);

    let _queryable_get_clock =
        declare_queryable!("fmi3GetClock", queryable_handlers::handle_get_clock);

    // Get/Set Binary
    let _queryable_set_binary =
        declare_queryable!("fmi3SetBinary", queryable_handlers::handle_set_binary);

    let _queryable_get_binary =
        declare_queryable!("fmi3GetBinary", queryable_handlers::handle_get_binary);

    // Reset
    let _queryable_reset = declare_queryable!("fmi3Reset", queryable_handlers::handle_reset);

    // Terminate
    let _queryable_terminate =
        declare_queryable!("fmi3Terminate", queryable_handlers::handle_terminate);

    info!("All queryables declared successfully");

    //=========================================================================
    // 6. Run Server Loop
    //=========================================================================

    info!("Liaison server is now listening!");
    info!("Press Ctrl+C to quit...");

    // Set up Ctrl+C handler
    let running = Arc::new(Mutex::new(true));
    let running_clone = Arc::clone(&running);

    ctrlc::set_handler(move || {
        info!("Received Ctrl+C, shutting down...");
        let mut running = running_clone.lock().unwrap();
        *running = false;
    })
    .context("Failed to set Ctrl+C handler")?;

    // Keep server running until Ctrl+C
    while *running.lock().unwrap() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    //=========================================================================
    // 7. Cleanup
    //=========================================================================

    info!("Shutting down server...");

    debug!("Cleaning up publishers...");
    drop(log_publisher);

    debug!("Closing Zenoh session...");
    if let Err(e) = session.close().wait() {
        error!("Error closing Zenoh session: {:?}", e);
    }

    debug!("Cleanup complete");
    info!("Server shutdown complete");

    Ok(())
}
