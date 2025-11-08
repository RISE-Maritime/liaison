//! Zenoh Queryable Handlers for FMI 3.0 Functions
//!
//! This module implements Zenoh query handlers for FMI 3.0 functions. Each handler:
//! 1. Parses the protobuf-encoded request from the query payload
//! 2. Calls the corresponding FMI function via the loaded FMU library
//! 3. Serializes the protobuf response and replies to the query
//!
//! # Handler Pattern
//!
//! All handlers follow a consistent pattern based on the C++ implementation in liaison.cpp:
//!
//! ```text
//! 1. Parse protobuf input from query.payload()
//! 2. Extract instance pointer from InstanceManager (if needed)
//! 3. Call FMU function via function pointer
//! 4. Create protobuf output message
//! 5. Serialize and reply to query
//! ```
//!
//! # Error Handling
//!
//! Handlers use comprehensive error handling with Result types and tracing for logging.
//! Errors are logged but handlers attempt to reply with error status when possible.
//!
//! # Thread Safety
//!
//! The InstanceManager is wrapped in Arc<Mutex<>> to ensure thread-safe access
//! across concurrent query handlers.
//!
//! # Implementation Notes
//!
//! Based on C++ implementation in src/liaison.cpp (lines 330-870):
//! - SetDebugLogging: lines 330-348
//! - InstantiateCoSimulation: lines 350-380
//! - InstantiateModelExchange: lines 382-404
//! - InstantiateScheduledExecution: lines 406-429
//! - EnterInitializationMode: lines 444-461
//! - ExitInitializationMode: lines 463-472
//! - FreeInstance: lines 475-495

use crate::callbacks::{fmi3_log_message, status_to_proto};
use crate::fmu_loader::{fmi3Status, FmuLibrary};
use crate::instance_manager::InstanceManager;
use crate::proto;
use anyhow::{Context, Result};
use prost::Message;
use std::ffi::CString;
use std::sync::Arc;
use tracing::{debug, error, info};
use zenoh::query::Query;
use zenoh::Wait;

/// Helper function to parse protobuf message from query payload
///
/// This function mirrors the C++ PARSE_QUERY macro from liaison.cpp:38-43
///
/// # Arguments
///
/// * `query` - The Zenoh query containing the protobuf payload
///
/// # Returns
///
/// A Result containing the parsed protobuf message or an error
///
/// # Example
///
/// ```ignore
/// let input: proto::Fmi3InstanceMessage = parse_query_payload(&query)?;
/// ```
fn parse_query_payload<T: Message + Default>(query: &Query) -> Result<T> {
    let payload = query
        .payload()
        .context("Query has no payload")?;

    let bytes = payload.to_bytes();

    T::decode(bytes.as_ref())
        .context("Failed to decode protobuf message from query payload")
}

/// Helper function to serialize protobuf message and reply to query
///
/// This function mirrors the C++ SERIALIZE_REPLY macro from liaison.cpp:45-49
///
/// # Arguments
///
/// * `query` - The Zenoh query to reply to
/// * `message` - The protobuf message to serialize and send
///
/// # Returns
///
/// A Result indicating success or failure of the reply operation
///
/// # Example
///
/// ```ignore
/// let output = proto::Fmi3StatusMessage { status: proto::Status::Ok as i32 };
/// serialize_and_reply(&query, &output)?;
/// ```
fn serialize_and_reply<T: Message>(query: &Query, message: &T) -> Result<()> {
    let mut buf = Vec::with_capacity(message.encoded_len());
    message.encode(&mut buf)
        .context("Failed to encode protobuf message")?;

    query
        .reply(query.key_expr().clone(), buf)
        .wait()
        .map_err(|e| anyhow::anyhow!("Failed to send query reply: {}", e))
}

/// Handler for fmi3SetDebugLogging
///
/// Enables or disables debug logging for an FMU instance with optional category filtering.
///
/// # Implementation Notes
///
/// Based on C++ implementation in liaison.cpp lines 330-348:
/// ```cpp
/// void fmi3SetDebugLogging(const zenoh::Query& query) {
///     proto::fmi3SetDebugLoggingMessage input;
///     PARSE_QUERY(query, input)
///     const char** categories = convertRepeatedFieldToCArray(input.categories());
///     fmi3Status status = fmu::fmi3SetDebugLogging(
///         getInstance(input.instance_index()),
///         input.logging_on(),
///         input.n_categories(),
///         categories
///     );
///     proto::fmi3StatusMessage output = makeFmi3StatusMessage(status);
///     SERIALIZE_REPLY(query, output)
/// }
/// ```
///
/// # Arguments
///
/// * `query` - The Zenoh query containing the request
/// * `fmu` - Reference to the loaded FMU library
/// * `instance_manager` - Thread-safe instance manager
/// * `resource_path` - Path to FMU resources (unused in this handler)
///
/// # Errors
///
/// Returns an error if:
/// - Query payload cannot be parsed
/// - Instance index is invalid
/// - String conversion fails
pub fn handle_set_debug_logging(
    query: Query,
    fmu: &FmuLibrary,
    instance_manager: Arc<InstanceManager>,
    _resource_path: &str,
) -> Result<()> {
    debug!("Handling fmi3SetDebugLogging query");

    // Parse input
    let input: proto::Fmi3SetDebugLoggingMessage = parse_query_payload(&query)?;

    // Get instance
    let instance = instance_manager
        .get_instance(input.instance_index)
        .context("Failed to get instance")?;

    // Convert categories to C strings
    let c_categories: Vec<CString> = input
        .categories
        .iter()
        .map(|s| CString::new(s.as_str()))
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("Failed to convert categories to C strings")?;

    let category_ptrs: Vec<*const i8> = c_categories
        .iter()
        .map(|cs| cs.as_ptr())
        .collect();

    // Call FMU function
    let status = (fmu.fmi3_set_debug_logging)(
        instance,
        input.logging_on as i32,
        input.n_categories as usize,
        category_ptrs.as_ptr(),
    );

    // Create response
    let output = proto::Fmi3StatusMessage {
        status: status_to_proto(status) as i32,
    };

    serialize_and_reply(&query, &output)?;

    info!(
        "fmi3SetDebugLogging completed with status: {:?}",
        status
    );

    Ok(())
}

/// Handler for fmi3InstantiateCoSimulation
///
/// Instantiates a new FMU instance for Co-Simulation.
///
/// # Implementation Notes
///
/// Based on C++ implementation in liaison.cpp lines 350-380:
/// ```cpp
/// void fmi3InstantiateCoSimulation(const zenoh::Query& query) {
///     proto::fmi3InstantiateCoSimulationMessage input;
///     PARSE_QUERY(query, input)
///     const fmi3ValueReference* required_intermediate_variables =
///         convertRepeatedFieldToCArray(input.required_intermediate_variables());
///     fmi3Instance instance = fmu::fmi3InstantiateCoSimulation(
///         input.instance_name().c_str(),
///         input.instantiation_token().c_str(),
///         *resourcePath,
///         input.visible(),
///         input.logging_on(),
///         input.event_mode_used(),
///         input.early_return_allowed(),
///         required_intermediate_variables,
///         input.n_required_intermediate_variables(),
///         nullptr,
///         fmi3LogMessage,
///         nullptr
///     );
///     proto::fmi3InstanceMessage output;
///     instances[nextIndex] = instance;
///     output.set_instance_index(nextIndex);
///     SERIALIZE_REPLY(query, output)
///     nextIndex++;
/// }
/// ```
///
/// # Arguments
///
/// * `query` - The Zenoh query containing the request
/// * `fmu` - Reference to the loaded FMU library
/// * `instance_manager` - Thread-safe instance manager
/// * `resource_path` - Path to FMU resources directory
pub fn handle_instantiate_co_simulation(
    query: Query,
    fmu: &FmuLibrary,
    instance_manager: Arc<InstanceManager>,
    resource_path: &str,
) -> Result<()> {
    debug!("Handling fmi3InstantiateCoSimulation query");

    // Parse input
    let input: proto::Fmi3InstantiateCoSimulationMessage = parse_query_payload(&query)?;

    // Convert strings to C strings
    let instance_name = CString::new(input.instance_name.as_str())
        .context("Failed to convert instance_name")?;
    let instantiation_token = CString::new(input.instantiation_token.as_str())
        .context("Failed to convert instantiation_token")?;
    let resource_path_c = CString::new(resource_path)
        .context("Failed to convert resource_path")?;

    // Convert required intermediate variables
    let required_intermediate_variables: Vec<u32> = input
        .required_intermediate_variables
        .iter()
        .map(|&x| x as u32)
        .collect();

    // Call FMU function
    let instance = (fmu.fmi3_instantiate_co_simulation)(
        instance_name.as_ptr(),
        instantiation_token.as_ptr(),
        resource_path_c.as_ptr(),
        input.visible as i32,
        input.logging_on as i32,
        input.event_mode_used as i32,
        input.early_return_allowed as i32,
        required_intermediate_variables.as_ptr(),
        input.n_required_intermediate_variables as usize,
        std::ptr::null_mut(),  // instance_environment
        Some(fmi3_log_message), // log_message callback
        None,                   // intermediate_update callback
    );

    // Check if instantiation failed
    if instance.is_null() {
        error!("fmi3InstantiateCoSimulation returned null instance");
        anyhow::bail!("FMU instantiation failed");
    }

    // Add instance to manager
    let instance_index = instance_manager.add_instance(instance);

    // Create response
    let output = proto::Fmi3InstanceMessage {
        instance_index,
    };

    serialize_and_reply(&query, &output)?;

    info!(
        "fmi3InstantiateCoSimulation completed, assigned index: {}",
        instance_index
    );

    Ok(())
}

/// Handler for fmi3InstantiateModelExchange
///
/// Instantiates a new FMU instance for Model Exchange.
///
/// # Implementation Notes
///
/// Based on C++ implementation in liaison.cpp lines 382-404:
/// ```cpp
/// void fmi3InstantiateModelExchange(const zenoh::Query& query) {
///     proto::fmi3InstantiateModelExchangeMessage input;
///     PARSE_QUERY(query, input)
///     fmi3Instance instance = fmu::fmi3InstantiateModelExchange(
///         input.instance_name().c_str(),
///         input.instantiation_token().c_str(),
///         *resourcePath,
///         input.visible(),
///         input.logging_on(),
///         nullptr,
///         fmi3LogMessage
///     );
///     proto::fmi3InstanceMessage output;
///     instances[nextIndex] = instance;
///     output.set_instance_index(nextIndex);
///     SERIALIZE_REPLY(query, output)
///     nextIndex++;
/// }
/// ```
pub fn handle_instantiate_model_exchange(
    query: Query,
    fmu: &FmuLibrary,
    instance_manager: Arc<InstanceManager>,
    resource_path: &str,
) -> Result<()> {
    debug!("Handling fmi3InstantiateModelExchange query");

    // Parse input
    let input: proto::Fmi3InstantiateModelExchangeMessage = parse_query_payload(&query)?;

    // Convert strings to C strings
    let instance_name = CString::new(input.instance_name.as_str())
        .context("Failed to convert instance_name")?;
    let instantiation_token = CString::new(input.instantiation_token.as_str())
        .context("Failed to convert instantiation_token")?;
    let resource_path_c = CString::new(resource_path)
        .context("Failed to convert resource_path")?;

    // Call FMU function
    let instance = (fmu.fmi3_instantiate_model_exchange)(
        instance_name.as_ptr(),
        instantiation_token.as_ptr(),
        resource_path_c.as_ptr(),
        input.visible as i32,
        input.logging_on as i32,
        std::ptr::null_mut(),  // instance_environment
        Some(fmi3_log_message), // log_message callback
    );

    // Check if instantiation failed
    if instance.is_null() {
        error!("fmi3InstantiateModelExchange returned null instance");
        anyhow::bail!("FMU instantiation failed");
    }

    // Add instance to manager
    let instance_index = instance_manager.add_instance(instance);

    // Create response
    let output = proto::Fmi3InstanceMessage {
        instance_index,
    };

    serialize_and_reply(&query, &output)?;

    info!(
        "fmi3InstantiateModelExchange completed, assigned index: {}",
        instance_index
    );

    Ok(())
}

/// Handler for fmi3InstantiateScheduledExecution
///
/// Instantiates a new FMU instance for Scheduled Execution.
///
/// # Implementation Notes
///
/// Based on C++ implementation in liaison.cpp lines 406-429:
/// ```cpp
/// void fmi3InstantiateScheduledExecution(const zenoh::Query& query) {
///     proto::fmi3InstantiateScheduledExecutionMessage input;
///     PARSE_QUERY(query, input)
///     fmi3Instance instance = fmu::fmi3InstantiateScheduledExecution(
///         input.instance_name().c_str(),
///         input.instantiation_token().c_str(),
///         *resourcePath,
///         input.visible(),
///         input.logging_on(),
///         nullptr,
///         fmi3LogMessage,
///         nullptr,
///         nullptr,
///         nullptr
///     );
///     proto::fmi3InstanceMessage output;
///     instances[nextIndex] = instance;
///     output.set_instance_index(nextIndex);
///     SERIALIZE_REPLY(query, output)
///     nextIndex++;
/// }
/// ```
pub fn handle_instantiate_scheduled_execution(
    query: Query,
    fmu: &FmuLibrary,
    instance_manager: Arc<InstanceManager>,
    resource_path: &str,
) -> Result<()> {
    debug!("Handling fmi3InstantiateScheduledExecution query");

    // Parse input
    let input: proto::Fmi3InstantiateScheduledExecutionMessage = parse_query_payload(&query)?;

    // Convert strings to C strings
    let instance_name = CString::new(input.instance_name.as_str())
        .context("Failed to convert instance_name")?;
    let instantiation_token = CString::new(input.instantiation_token.as_str())
        .context("Failed to convert instantiation_token")?;
    let resource_path_c = CString::new(resource_path)
        .context("Failed to convert resource_path")?;

    // Call FMU function
    let instance = (fmu.fmi3_instantiate_scheduled_execution)(
        instance_name.as_ptr(),
        instantiation_token.as_ptr(),
        resource_path_c.as_ptr(),
        input.visible as i32,
        input.logging_on as i32,
        std::ptr::null_mut(),  // instance_environment
        Some(fmi3_log_message), // log_message callback
        None,                   // clock_update callback
        None,                   // lock_preemption callback
        None,                   // unlock_preemption callback
    );

    // Check if instantiation failed
    if instance.is_null() {
        error!("fmi3InstantiateScheduledExecution returned null instance");
        anyhow::bail!("FMU instantiation failed");
    }

    // Add instance to manager
    let instance_index = instance_manager.add_instance(instance);

    // Create response
    let output = proto::Fmi3InstanceMessage {
        instance_index,
    };

    serialize_and_reply(&query, &output)?;

    info!(
        "fmi3InstantiateScheduledExecution completed, assigned index: {}",
        instance_index
    );

    Ok(())
}

/// Handler for fmi3EnterInitializationMode
///
/// Enters initialization mode for an FMU instance.
///
/// # Implementation Notes
///
/// Based on C++ implementation in liaison.cpp lines 444-461:
/// ```cpp
/// void fmi3EnterInitializationMode(const zenoh::Query& query) {
///     proto::fmi3EnterInitializationModeMessage input;
///     PARSE_QUERY(query, input)
///     fmi3Status status = fmu::fmi3EnterInitializationMode(
///         getInstance(input.instance_index()),
///         input.tolerance_defined(),
///         input.tolerance(),
///         input.start_time(),
///         input.stop_time_defined(),
///         input.stop_time()
///     );
///     proto::fmi3StatusMessage output = makeFmi3StatusMessage(status);
///     SERIALIZE_REPLY(query, output)
/// }
/// ```
pub fn handle_enter_initialization_mode(
    query: Query,
    fmu: &FmuLibrary,
    instance_manager: Arc<InstanceManager>,
    _resource_path: &str,
) -> Result<()> {
    debug!("Handling fmi3EnterInitializationMode query");

    // Parse input
    let input: proto::Fmi3EnterInitializationModeMessage = parse_query_payload(&query)?;

    // Get instance
    let instance = instance_manager
        .get_instance(input.instance_index)
        .context("Failed to get instance")?;

    // Call FMU function
    let status = (fmu.fmi3_enter_initialization_mode)(
        instance,
        input.tolerance_defined as i32,
        input.tolerance,
        input.start_time,
        input.stop_time_defined as i32,
        input.stop_time,
    );

    // Create response
    let output = proto::Fmi3StatusMessage {
        status: status_to_proto(status) as i32,
    };

    serialize_and_reply(&query, &output)?;

    info!(
        "fmi3EnterInitializationMode completed with status: {:?}",
        status
    );

    Ok(())
}

/// Handler for fmi3ExitInitializationMode
///
/// Exits initialization mode for an FMU instance.
///
/// # Implementation Notes
///
/// Based on C++ implementation in liaison.cpp lines 463-472:
/// ```cpp
/// void fmi3ExitInitializationMode(const zenoh::Query& query) {
///     proto::fmi3InstanceMessage input;
///     PARSE_QUERY(query, input)
///     fmi3Status status = fmu::fmi3ExitInitializationMode(getInstance(input.instance_index()));
///     proto::fmi3StatusMessage output = makeFmi3StatusMessage(status);
///     SERIALIZE_REPLY(query, output)
/// }
/// ```
pub fn handle_exit_initialization_mode(
    query: Query,
    fmu: &FmuLibrary,
    instance_manager: Arc<InstanceManager>,
    _resource_path: &str,
) -> Result<()> {
    debug!("Handling fmi3ExitInitializationMode query");

    // Parse input
    let input: proto::Fmi3InstanceMessage = parse_query_payload(&query)?;

    // Get instance
    let instance = instance_manager
        .get_instance(input.instance_index)
        .context("Failed to get instance")?;

    // Call FMU function
    let status = (fmu.fmi3_exit_initialization_mode)(instance);

    // Create response
    let output = proto::Fmi3StatusMessage {
        status: status_to_proto(status) as i32,
    };

    serialize_and_reply(&query, &output)?;

    info!(
        "fmi3ExitInitializationMode completed with status: {:?}",
        status
    );

    Ok(())
}

/// Handler for fmi3EnterEventMode
///
/// Switches the FMU instance to event mode for handling discrete events.
///
/// # Implementation Notes
///
/// Based on C++ implementation in liaison.cpp lines 431-441:
/// ```cpp
/// void fmi3EnterEventMode(const zenoh::Query& query) {
///     printQuery(query);
///     proto::fmi3InstanceMessage input;
///     PARSE_QUERY(query, input)
///     fmi3Status status = fmu::fmi3EnterEventMode(getInstance(input.instance_index()));
///     proto::fmi3StatusMessage output = makeFmi3StatusMessage(status);
///     SERIALIZE_REPLY(query, output)
/// }
/// ```
pub fn handle_enter_event_mode(
    query: Query,
    fmu: &FmuLibrary,
    instance_manager: Arc<InstanceManager>,
    _resource_path: &str,
) -> Result<()> {
    debug!("Handling fmi3EnterEventMode query");

    // Parse input
    let input: proto::Fmi3InstanceMessage = parse_query_payload(&query)?;

    // Get instance
    let instance = instance_manager
        .get_instance(input.instance_index)
        .context("Failed to get instance")?;

    // Call FMU function
    let status = (fmu.fmi3_enter_event_mode)(instance);

    // Create response
    let output = proto::Fmi3StatusMessage {
        status: status_to_proto(status) as i32,
    };

    serialize_and_reply(&query, &output)?;

    info!("fmi3EnterEventMode completed with status: {:?}", status);

    Ok(())
}

/// Handler for fmi3FreeInstance
///
/// Frees an FMU instance and removes it from the instance manager.
///
/// # Implementation Notes
///
/// Based on C++ implementation in liaison.cpp lines 475-495:
/// ```cpp
/// void fmi3FreeInstance(const zenoh::Query& query) {
///     proto::fmi3InstanceMessage input;
///     PARSE_QUERY(query, input)
///     try {
///         fmu::fmi3FreeInstance(getInstance(input.instance_index()));
///     } catch (std::runtime_error& error) {
///         spdlog::error("Failed to free FMU instance.");
///     }
///     try {
///         auto it = instances.find(input.instance_index());
///         instances.erase(it);
///     } catch (std::runtime_error& error) {
///         spdlog::error("Failed to erase instance from instances.");
///     }
///     proto::voidMessage output;
///     SERIALIZE_REPLY(query, output)
/// }
/// ```
///
/// # Error Handling
///
/// This handler attempts to free the instance and remove it from the manager.
/// Errors are logged but do not prevent the response from being sent.
pub fn handle_free_instance(
    query: Query,
    fmu: &FmuLibrary,
    instance_manager: Arc<InstanceManager>,
    _resource_path: &str,
) -> Result<()> {
    debug!("Handling fmi3FreeInstance query");

    // Parse input
    let input: proto::Fmi3InstanceMessage = parse_query_payload(&query)?;

    // Try to get and free the instance
    match instance_manager.get_instance(input.instance_index) {
        Ok(instance) => {
            // Call FMU function to free the instance
            (fmu.fmi3_free_instance)(instance);
            info!("Successfully freed FMU instance {}", input.instance_index);
        }
        Err(e) => {
            error!(
                "Failed to get instance {} for freeing: {}",
                input.instance_index, e
            );
            // Continue to try removing from manager anyway
        }
    }

    // Try to remove the instance from the manager
    match instance_manager.remove_instance(input.instance_index) {
        Ok(_) => {
            info!(
                "Successfully removed instance {} from manager",
                input.instance_index
            );
        }
        Err(e) => {
            error!(
                "Failed to remove instance {} from manager: {}",
                input.instance_index, e
            );
        }
    }

    // Create response (void message)
    let output = proto::VoidMessage {};

    serialize_and_reply(&query, &output)?;

    info!("fmi3FreeInstance completed for instance {}", input.instance_index);

    Ok(())
}

//=============================================================================
// Additional Lifecycle Handlers
//=============================================================================

/// Handler for fmi3Terminate
///
/// Terminates the FMU instance, indicating that the simulation run is complete.
///
/// # Implementation Notes
///
/// Based on liaison.cpp lines 729-739:
/// ```cpp
/// void fmi3Terminate(const zenoh::Query& query) {
///     printQuery(query);
///     proto::fmi3InstanceMessage input;
///     PARSE_QUERY(query, input)
///     fmi3Status status = fmu::fmi3Terminate(getInstance(input.instance_index()));
///     proto::fmi3StatusMessage output = makeFmi3StatusMessage(status);
///     SERIALIZE_REPLY(query, output)
/// }
/// ```
pub fn handle_terminate(
    query: Query,
    fmu: &FmuLibrary,
    instance_manager: Arc<InstanceManager>,
    _resource_path: &str,
) -> Result<()> {
    debug!("Handling fmi3Terminate query");

    // Parse input
    let input: proto::Fmi3InstanceMessage = parse_query_payload(&query)?;

    // Get instance
    let instance = instance_manager
        .get_instance(input.instance_index)
        .context("Failed to get instance")?;

    // Call FMU function
    let status = (fmu.fmi3_terminate)(instance);

    // Create response
    let output = proto::Fmi3StatusMessage {
        status: status_to_proto(status) as i32,
    };

    serialize_and_reply(&query, &output)?;

    info!(
        "fmi3Terminate completed with status: {:?}",
        status
    );

    Ok(())
}

/// Handler for fmi3Reset
///
/// Resets the FMU instance to its initial state (after instantiation).
///
/// # Implementation Notes
///
/// Based on liaison.cpp lines 716-726:
/// ```cpp
/// void fmi3Reset(const zenoh::Query& query) {
///     printQuery(query);
///     proto::fmi3InstanceMessage input;
///     PARSE_QUERY(query, input)
///     fmi3Status status = fmu::fmi3Reset(getInstance(input.instance_index()));
///     proto::fmi3StatusMessage output = makeFmi3StatusMessage(status);
///     SERIALIZE_REPLY(query, output)
/// }
/// ```
pub fn handle_reset(
    query: Query,
    fmu: &FmuLibrary,
    instance_manager: Arc<InstanceManager>,
    _resource_path: &str,
) -> Result<()> {
    debug!("Handling fmi3Reset query");

    // Parse input
    let input: proto::Fmi3InstanceMessage = parse_query_payload(&query)?;

    // Get instance
    let instance = instance_manager
        .get_instance(input.instance_index)
        .context("Failed to get instance")?;

    // Call FMU function
    let status = (fmu.fmi3_reset)(instance);

    // Create response
    let output = proto::Fmi3StatusMessage {
        status: status_to_proto(status) as i32,
    };

    serialize_and_reply(&query, &output)?;

    info!(
        "fmi3Reset completed with status: {:?}",
        status
    );

    Ok(())
}

//=============================================================================
// Co-Simulation Handlers
//=============================================================================

/// Handler for fmi3DoStep
///
/// Performs a co-simulation step from the current communication point to
/// the next communication point.
///
/// # Implementation Notes
///
/// Based on liaison.cpp lines 498-521:
/// ```cpp
/// void fmi3DoStep(const zenoh::Query& query) {
///     printQuery(query);
///     proto::fmi3DoStepMessage input;
///     PARSE_QUERY(query, input)
///     fmi3Boolean event_handling_needed = input.event_handling_needed();
///     fmi3Boolean terminate_simulation = input.terminate_simulation();
///     fmi3Boolean early_return = input.early_return();
///     fmi3Float64 last_successful_time = input.last_successful_time();
///     fmi3Status status = fmu::fmi3DoStep(
///         getInstance(input.instance_index()),
///         input.current_communication_point(),
///         input.communication_step_size(),
///         input.no_set_fmu_state_prior_to_current_point(),
///         &event_handling_needed,
///         &terminate_simulation,
///         &early_return,
///         &last_successful_time
///     );
///     proto::fmi3StatusMessage output = makeFmi3StatusMessage(status);
///     SERIALIZE_REPLY(query, output)
/// }
/// ```
pub fn handle_do_step(
    query: Query,
    fmu: &FmuLibrary,
    instance_manager: Arc<InstanceManager>,
    _resource_path: &str,
) -> Result<()> {
    debug!("Handling fmi3DoStep query");

    // Parse input
    let input: proto::Fmi3DoStepMessage = parse_query_payload(&query)?;

    // Get instance
    let instance = instance_manager
        .get_instance(input.instance_index)
        .context("Failed to get instance")?;

    // Prepare mutable output parameters
    let mut event_handling_needed = input.event_handling_needed as i32;
    let mut terminate_simulation = input.terminate_simulation as i32;
    let mut early_return = input.early_return as i32;
    let mut last_successful_time = input.last_successful_time;

    // Call FMU function
    let status = (fmu.fmi3_do_step)(
        instance,
        input.current_communication_point,
        input.communication_step_size,
        input.no_set_fmu_state_prior_to_current_point as i32,
        &mut event_handling_needed,
        &mut terminate_simulation,
        &mut early_return,
        &mut last_successful_time,
    );

    // Create response
    let output = proto::Fmi3StatusMessage {
        status: status_to_proto(status) as i32,
    };

    serialize_and_reply(&query, &output)?;

    info!(
        "fmi3DoStep completed with status: {:?}",
        status
    );

    Ok(())
}

//=============================================================================
// Macro-Based Get/Set Value Handlers
//=============================================================================

/// Macro to generate fmi3Get{Type} handler functions
///
/// Generates handler functions for getting values from the FMU.
/// The macro handles the common pattern:
/// 1. Parse input message
/// 2. Extract value references array
/// 3. Call FMU get function
/// 4. Pack results into output message
/// 5. Reply with serialized output
///
/// # Arguments
///
/// * `$handler_name` - The handler function name
/// * `$type_name` - The FMI type name for logging (e.g., "Float32", "Int16")
/// * `$proto_input` - The protobuf input message type
/// * `$proto_output` - The protobuf output message type
/// * `$rust_type` - The Rust type for values (e.g., f32, i16)
/// * `$fmi_fn` - The FMU library function field name
///
/// # Implementation Notes
///
/// Based on liaison.cpp lines 66-96 (DEFINE_FMI3_GET_VALUE_FUNCTION macro)
macro_rules! impl_get_value_handler {
    ($handler_name:ident, $type_name:expr, $proto_input:ty, $proto_output:ident, $rust_type:ty, $proto_type:ty, $fmi_fn:ident) => {
        pub fn $handler_name(
            query: Query,
            fmu: &FmuLibrary,
            instance_manager: Arc<InstanceManager>,
            _resource_path: &str,
        ) -> Result<()> {
            debug!(concat!("Handling fmi3Get", $type_name, " query"));

            // Parse input
            let input: $proto_input = parse_query_payload(&query)?;

            // Get instance
            let instance = instance_manager
                .get_instance(input.instance_index)
                .context("Failed to get instance")?;

            // Extract value references
            let value_references: Vec<u32> = input
                .value_references
                .iter()
                .map(|&vr| vr as u32)
                .collect();

            // Prepare output buffer
            let n_values = input.n_value_references as usize;
            let mut values: Vec<$rust_type> = vec![Default::default(); n_values];
            let n_values_out = n_values;

            // Call FMU function
            let status = (fmu.$fmi_fn)(
                instance,
                value_references.as_ptr(),
                n_values,
                values.as_mut_ptr(),
                n_values_out,
            );

            // Convert values to proto type
            let proto_values: Vec<$proto_type> = values.iter().map(|&v| v as $proto_type).collect();

            // Create output
            let output = proto::$proto_output {
                values: proto_values,
                n_values: n_values_out as i32,
                status: status_to_proto(status) as i32,
            };

            serialize_and_reply(&query, &output)?;

            info!(concat!("fmi3Get", $type_name, " completed with status: {:?}"), status);

            Ok(())
        }
    };
}

/// Macro to generate fmi3Set{Type} handler functions
///
/// Generates handler functions for setting values in the FMU.
/// The macro handles the common pattern:
/// 1. Parse input message
/// 2. Extract value references and values arrays
/// 3. Call FMU set function
/// 4. Reply with status message
///
/// # Arguments
///
/// * `$handler_name` - The handler function name
/// * `$type_name` - The FMI type name for logging (e.g., "Float32", "Int16")
/// * `$proto_input` - The protobuf input message type
/// * `$rust_type` - The Rust type for values (e.g., f32, i16)
/// * `$fmi_fn` - The FMU library function field name
///
/// # Implementation Notes
///
/// Based on liaison.cpp lines 98-124 (DEFINE_FMI3_SET_VALUE_FUNCTION macro)
macro_rules! impl_set_value_handler {
    ($handler_name:ident, $type_name:expr, $proto_input:ty, $rust_type:ty, $proto_type:ty, $fmi_fn:ident) => {
        pub fn $handler_name(
            query: Query,
            fmu: &FmuLibrary,
            instance_manager: Arc<InstanceManager>,
            _resource_path: &str,
        ) -> Result<()> {
            debug!(concat!("Handling fmi3Set", $type_name, " query"));

            // Parse input
            let input: $proto_input = parse_query_payload(&query)?;

            // Get instance
            let instance = instance_manager
                .get_instance(input.instance_index)
                .context("Failed to get instance")?;

            // Extract value references
            let value_references: Vec<u32> = input
                .value_references
                .iter()
                .map(|&vr| vr as u32)
                .collect();

            // Convert values from proto type to FMI type
            let values: Vec<$rust_type> = input.values.iter().map(|&v| v as $rust_type).collect();

            // Call FMU function
            let status = (fmu.$fmi_fn)(
                instance,
                value_references.as_ptr(),
                input.n_value_references as usize,
                values.as_ptr(),
                input.n_values as usize,
            );

            // Create output
            let output = proto::Fmi3StatusMessage {
                status: status_to_proto(status) as i32,
            };

            serialize_and_reply(&query, &output)?;

            info!(concat!("fmi3Set", $type_name, " completed with status: {:?}"), status);

            Ok(())
        }
    };
}

//=============================================================================
// Generated Get Value Handlers for All FMI Types
//=============================================================================

// Float types
impl_get_value_handler!(
    handle_get_float32,
    "Float32",
    proto::Fmi3GetFloat32InputMessage,
    Fmi3GetFloat32OutputMessage,
    f32,
    f32,
    fmi3_get_float32
);

impl_get_value_handler!(
    handle_get_float64,
    "Float64",
    proto::Fmi3GetFloat64InputMessage,
    Fmi3GetFloat64OutputMessage,
    f64,
    f64,
    fmi3_get_float64
);

// Signed integer types
impl_get_value_handler!(
    handle_get_int8,
    "Int8",
    proto::Fmi3GetInt8InputMessage,
    Fmi3GetInt8OutputMessage,
    i8,
    i32,
    fmi3_get_int8
);

impl_get_value_handler!(
    handle_get_int16,
    "Int16",
    proto::Fmi3GetInt16InputMessage,
    Fmi3GetInt16OutputMessage,
    i16,
    i32,
    fmi3_get_int16
);

impl_get_value_handler!(
    handle_get_int32,
    "Int32",
    proto::Fmi3GetInt32InputMessage,
    Fmi3GetInt32OutputMessage,
    i32,
    i32,
    fmi3_get_int32
);

impl_get_value_handler!(
    handle_get_int64,
    "Int64",
    proto::Fmi3GetInt64InputMessage,
    Fmi3GetInt64OutputMessage,
    i64,
    i64,
    fmi3_get_int64
);

// Unsigned integer types
impl_get_value_handler!(
    handle_get_uint8,
    "UInt8",
    proto::Fmi3GetUInt8InputMessage,
    Fmi3GetUInt8OutputMessage,
    u8,
    u32,
    fmi3_get_uint8
);

impl_get_value_handler!(
    handle_get_uint16,
    "UInt16",
    proto::Fmi3GetUInt16InputMessage,
    Fmi3GetUInt16OutputMessage,
    u16,
    u32,
    fmi3_get_uint16
);

impl_get_value_handler!(
    handle_get_uint32,
    "UInt32",
    proto::Fmi3GetUInt32InputMessage,
    Fmi3GetUInt32OutputMessage,
    u32,
    u32,
    fmi3_get_uint32
);

impl_get_value_handler!(
    handle_get_uint64,
    "UInt64",
    proto::Fmi3GetUInt64InputMessage,
    Fmi3GetUInt64OutputMessage,
    u64,
    u64,
    fmi3_get_uint64
);

// Boolean type - special handler because FMI uses i32 for boolean
pub fn handle_get_boolean(
    query: Query,
    fmu: &FmuLibrary,
    instance_manager: Arc<InstanceManager>,
    _resource_path: &str,
) -> Result<()> {
    debug!("Handling fmi3GetBoolean query");

    // Parse input
    let input: proto::Fmi3GetBooleanInputMessage = parse_query_payload(&query)?;

    // Get instance
    let instance = instance_manager
        .get_instance(input.instance_index)
        .context("Failed to get instance")?;

    // Extract value references
    let value_references: Vec<u32> = input
        .value_references
        .iter()
        .map(|&vr| vr as u32)
        .collect();

    // Prepare output buffer - FMI uses i32 for boolean
    let n_values = input.n_value_references as usize;
    let mut values: Vec<i32> = vec![0; n_values];
    let n_values_out = n_values;

    // Call FMU function
    let status = (fmu.fmi3_get_boolean)(
        instance,
        value_references.as_ptr(),
        n_values,
        values.as_mut_ptr(),
        n_values_out,
    );

    // Convert i32 values to bool for protobuf
    let proto_values: Vec<bool> = values.iter().map(|&v| v != 0).collect();

    // Create output
    let output = proto::Fmi3GetBooleanOutputMessage {
        values: proto_values,
        n_values: n_values_out as i32,
        status: status_to_proto(status) as i32,
    };

    serialize_and_reply(&query, &output)?;

    info!("fmi3GetBoolean completed with status: {:?}", status);

    Ok(())
}

//=============================================================================
// Generated Set Value Handlers for All FMI Types
//=============================================================================

// Float types
impl_set_value_handler!(
    handle_set_float32,
    "Float32",
    proto::Fmi3SetFloat32InputMessage,
    f32,
    f32,
    fmi3_set_float32
);

impl_set_value_handler!(
    handle_set_float64,
    "Float64",
    proto::Fmi3SetFloat64InputMessage,
    f64,
    f64,
    fmi3_set_float64
);

// Signed integer types
impl_set_value_handler!(
    handle_set_int8,
    "Int8",
    proto::Fmi3SetInt8InputMessage,
    i8,
    i32,
    fmi3_set_int8
);

impl_set_value_handler!(
    handle_set_int16,
    "Int16",
    proto::Fmi3SetInt16InputMessage,
    i16,
    i32,
    fmi3_set_int16
);

impl_set_value_handler!(
    handle_set_int32,
    "Int32",
    proto::Fmi3SetInt32InputMessage,
    i32,
    i32,
    fmi3_set_int32
);

impl_set_value_handler!(
    handle_set_int64,
    "Int64",
    proto::Fmi3SetInt64InputMessage,
    i64,
    i64,
    fmi3_set_int64
);

// Unsigned integer types
impl_set_value_handler!(
    handle_set_uint8,
    "UInt8",
    proto::Fmi3SetUInt8InputMessage,
    u8,
    u32,
    fmi3_set_uint8
);

impl_set_value_handler!(
    handle_set_uint16,
    "UInt16",
    proto::Fmi3SetUInt16InputMessage,
    u16,
    u32,
    fmi3_set_uint16
);

impl_set_value_handler!(
    handle_set_uint32,
    "UInt32",
    proto::Fmi3SetUInt32InputMessage,
    u32,
    u32,
    fmi3_set_uint32
);

impl_set_value_handler!(
    handle_set_uint64,
    "UInt64",
    proto::Fmi3SetUInt64InputMessage,
    u64,
    u64,
    fmi3_set_uint64
);

// Boolean type - special handler because FMI uses i32 for boolean
pub fn handle_set_boolean(
    query: Query,
    fmu: &FmuLibrary,
    instance_manager: Arc<InstanceManager>,
    _resource_path: &str,
) -> Result<()> {
    debug!("Handling fmi3SetBoolean query");

    // Parse input
    let input: proto::Fmi3SetBooleanInputMessage = parse_query_payload(&query)?;

    // Get instance
    let instance = instance_manager
        .get_instance(input.instance_index)
        .context("Failed to get instance")?;

    // Extract value references
    let value_references: Vec<u32> = input
        .value_references
        .iter()
        .map(|&vr| vr as u32)
        .collect();

    // Convert bool values to i32 for FMI (true -> 1, false -> 0)
    let values: Vec<i32> = input.values.iter().map(|&b| if b { 1 } else { 0 }).collect();

    // Call FMU function
    let status = (fmu.fmi3_set_boolean)(
        instance,
        value_references.as_ptr(),
        input.n_value_references as usize,
        values.as_ptr(),
        input.n_values as usize,
    );

    // Create output
    let output = proto::Fmi3StatusMessage {
        status: status_to_proto(status) as i32,
    };

    serialize_and_reply(&query, &output)?;

    info!("fmi3SetBoolean completed with status: {:?}", status);

    Ok(())
}

//=============================================================================
// Special Handlers for String, Binary, and Clock Types
//=============================================================================

/// Handler for fmi3GetString
///
/// Gets string values from the FMU. Requires special handling to convert
/// between C strings and Rust strings.
///
/// # Implementation Notes
///
/// Based on liaison.cpp line 583 (DEFINE_FMI3_GET_VALUE_FUNCTION(String))
/// with special C string handling
pub fn handle_get_string(
    query: Query,
    fmu: &FmuLibrary,
    instance_manager: Arc<InstanceManager>,
    _resource_path: &str,
) -> Result<()> {
    use std::ffi::CStr;

    debug!("Handling fmi3GetString query");

    // Parse input
    let input: proto::Fmi3GetStringInputMessage = parse_query_payload(&query)?;

    // Get instance
    let instance = instance_manager
        .get_instance(input.instance_index)
        .context("Failed to get instance")?;

    // Extract value references
    let value_references: Vec<u32> = input
        .value_references
        .iter()
        .map(|&vr| vr as u32)
        .collect();

    // Prepare output buffer for C string pointers
    let n_values = input.n_value_references as usize;
    let mut c_string_ptrs: Vec<*const i8> = vec![std::ptr::null(); n_values];
    let n_values_out = n_values;

    // Call FMU function
    let status = (fmu.fmi3_get_string)(
        instance,
        value_references.as_ptr(),
        n_values,
        c_string_ptrs.as_mut_ptr(),
        n_values_out,
    );

    // Convert C strings to Rust strings
    let mut rust_strings: Vec<String> = Vec::new();
    for &c_str_ptr in c_string_ptrs.iter().take(n_values_out) {
        if c_str_ptr.is_null() {
            rust_strings.push(String::new());
        } else {
            unsafe {
                let c_str = CStr::from_ptr(c_str_ptr);
                rust_strings.push(c_str.to_string_lossy().into_owned());
            }
        }
    }

    // Create output
    let output = proto::Fmi3GetStringOutputMessage {
        values: rust_strings,
        n_values: n_values_out as i32,
        status: status_to_proto(status) as i32,
    };

    serialize_and_reply(&query, &output)?;

    info!("fmi3GetString completed with status: {:?}", status);

    Ok(())
}

/// Handler for fmi3SetString
///
/// Sets string values in the FMU. Requires special handling to convert
/// from Rust strings to C strings.
///
/// # Implementation Notes
///
/// Based on liaison.cpp lines 556-582
pub fn handle_set_string(
    query: Query,
    fmu: &FmuLibrary,
    instance_manager: Arc<InstanceManager>,
    _resource_path: &str,
) -> Result<()> {
    debug!("Handling fmi3SetString query");

    // Parse input
    let input: proto::Fmi3SetStringInputMessage = parse_query_payload(&query)?;

    // Get instance
    let instance = instance_manager
        .get_instance(input.instance_index)
        .context("Failed to get instance")?;

    // Extract value references
    let value_references: Vec<u32> = input
        .value_references
        .iter()
        .map(|&vr| vr as u32)
        .collect();

    // Convert Rust strings to C strings
    // We need to keep the CStrings alive until after the FMU call
    let c_strings: Vec<CString> = input
        .values
        .iter()
        .map(|s| CString::new(s.as_str()).unwrap_or_default())
        .collect();

    let c_string_ptrs: Vec<*const i8> = c_strings
        .iter()
        .map(|cs| cs.as_ptr())
        .collect();

    // Call FMU function
    let status = (fmu.fmi3_set_string)(
        instance,
        value_references.as_ptr(),
        input.n_value_references as usize,
        c_string_ptrs.as_ptr(),
        input.n_values as usize,
    );

    // Create output
    let output = proto::Fmi3StatusMessage {
        status: status_to_proto(status) as i32,
    };

    serialize_and_reply(&query, &output)?;

    info!("fmi3SetString completed with status: {:?}", status);

    Ok(())
}

/// Handler for fmi3GetBinary
///
/// Gets binary values from the FMU. Requires special handling for size arrays
/// and byte buffers.
///
/// # Implementation Notes
///
/// Based on liaison.cpp lines 677-713
pub fn handle_get_binary(
    query: Query,
    fmu: &FmuLibrary,
    instance_manager: Arc<InstanceManager>,
    _resource_path: &str,
) -> Result<()> {
    debug!("Handling fmi3GetBinary query");

    // Parse input
    let input: proto::Fmi3GetBinaryInputMessage = parse_query_payload(&query)?;

    // Get instance
    let instance = instance_manager
        .get_instance(input.instance_index)
        .context("Failed to get instance")?;

    // Extract value references
    let value_references: Vec<u32> = input
        .value_references
        .iter()
        .map(|&vr| vr as u32)
        .collect();

    let n_value_references = input.n_value_references as usize;

    // Prepare buffers
    // Allocate a large buffer for binary data (MAX_BINARY_SIZE per reference)
    const MAX_BINARY_SIZE: usize = 4096;
    let mut value_sizes: Vec<usize> = vec![0; n_value_references];
    let binary_buffer: Vec<u8> = vec![0; n_value_references * MAX_BINARY_SIZE];
    let mut binary_ptrs: Vec<*const u8> = vec![std::ptr::null(); n_value_references];

    // Set up pointers to buffer segments
    for i in 0..n_value_references {
        binary_ptrs[i] = unsafe { binary_buffer.as_ptr().add(i * MAX_BINARY_SIZE) };
    }

    let n_values_out = 0;

    // Call FMU function
    let status = (fmu.fmi3_get_binary)(
        instance,
        value_references.as_ptr(),
        n_value_references,
        value_sizes.as_mut_ptr(),
        binary_ptrs.as_mut_ptr(),
        n_values_out,
    );

    // Extract binary data into separate byte arrays
    let mut output_values: Vec<Vec<u8>> = Vec::new();
    let mut offset = 0;
    for &size in value_sizes.iter().take(n_value_references) {
        let binary_data = binary_buffer[offset..offset + size].to_vec();
        output_values.push(binary_data);
        offset += size;
    }

    // Create output
    let output = proto::Fmi3GetBinaryOutputMessage {
        values: output_values,
        n_values: n_value_references as i32,
        status: status_to_proto(status) as i32,
    };

    serialize_and_reply(&query, &output)?;

    info!("fmi3GetBinary completed with status: {:?}", status);

    Ok(())
}

/// Handler for fmi3SetBinary
///
/// Sets binary values in the FMU. Requires special handling for size arrays
/// and byte buffers.
///
/// # Implementation Notes
///
/// Based on liaison.cpp lines 642-673
pub fn handle_set_binary(
    query: Query,
    fmu: &FmuLibrary,
    instance_manager: Arc<InstanceManager>,
    _resource_path: &str,
) -> Result<()> {
    debug!("Handling fmi3SetBinary query");

    // Parse input
    let input: proto::Fmi3SetBinaryInputMessage = parse_query_payload(&query)?;

    // Get instance
    let instance = instance_manager
        .get_instance(input.instance_index)
        .context("Failed to get instance")?;

    // Extract value references
    let value_references: Vec<u32> = input
        .value_references
        .iter()
        .map(|&vr| vr as u32)
        .collect();

    let n_value_references = input.n_value_references as usize;

    // Prepare size array and concatenated binary buffer
    let mut value_sizes: Vec<usize> = Vec::new();
    let mut binary_buffer: Vec<u8> = Vec::new();

    for binary_value in input.values.iter() {
        value_sizes.push(binary_value.len());
        binary_buffer.extend_from_slice(binary_value);
    }

    // Create pointers array for FMI function
    let mut binary_ptrs: Vec<*const u8> = Vec::new();
    let mut offset = 0;
    for &size in value_sizes.iter() {
        binary_ptrs.push(unsafe { binary_buffer.as_ptr().add(offset) });
        offset += size;
    }

    // Call FMU function
    let status = (fmu.fmi3_set_binary)(
        instance,
        value_references.as_ptr(),
        n_value_references,
        value_sizes.as_ptr(),
        binary_ptrs.as_ptr(),
        binary_buffer.len(),
    );

    // Create output
    let output = proto::Fmi3StatusMessage {
        status: status_to_proto(status) as i32,
    };

    serialize_and_reply(&query, &output)?;

    info!("fmi3SetBinary completed with status: {:?}", status);

    Ok(())
}

/// Handler for fmi3GetClock
///
/// Gets clock values from the FMU.
///
/// # Implementation Notes
///
/// Based on liaison.cpp lines 609-638
pub fn handle_get_clock(
    query: Query,
    fmu: &FmuLibrary,
    instance_manager: Arc<InstanceManager>,
    _resource_path: &str,
) -> Result<()> {
    debug!("Handling fmi3GetClock query");

    // Parse input
    let input: proto::Fmi3GetClockInputMessage = parse_query_payload(&query)?;

    // Get instance
    let instance = instance_manager
        .get_instance(input.instance_index)
        .context("Failed to get instance")?;

    // Extract value references
    let value_references: Vec<u32> = input
        .value_references
        .iter()
        .map(|&vr| vr as u32)
        .collect();

    // Prepare output buffer for clock values (i32 in FMI)
    let n_values = input.n_value_references as usize;
    let mut clock_values: Vec<i32> = vec![0; n_values];

    // Call FMU function
    let status = (fmu.fmi3_get_clock)(
        instance,
        value_references.as_ptr(),
        n_values,
        clock_values.as_mut_ptr(),
    );

    // Convert i32 clock values to bool for protobuf
    let bool_values: Vec<bool> = clock_values.iter().map(|&v| v != 0).collect();

    // Create output
    let output = proto::Fmi3GetClockOutputMessage {
        values: bool_values,
        n_values: n_values as i32,
        status: status_to_proto(status) as i32,
    };

    serialize_and_reply(&query, &output)?;

    info!("fmi3GetClock completed with status: {:?}", status);

    Ok(())
}

/// Handler for fmi3SetClock
///
/// Sets clock values in the FMU.
///
/// # Implementation Notes
///
/// Based on liaison.cpp lines 585-607
pub fn handle_set_clock(
    query: Query,
    fmu: &FmuLibrary,
    instance_manager: Arc<InstanceManager>,
    _resource_path: &str,
) -> Result<()> {
    debug!("Handling fmi3SetClock query");

    // Parse input
    let input: proto::Fmi3SetClockInputMessage = parse_query_payload(&query)?;

    // Get instance
    let instance = instance_manager
        .get_instance(input.instance_index)
        .context("Failed to get instance")?;

    // Extract value references
    let value_references: Vec<u32> = input
        .value_references
        .iter()
        .map(|&vr| vr as u32)
        .collect();

    // Convert bool values to i32 for FMI
    let clock_values: Vec<i32> = input.values.iter().map(|&b| if b { 1 } else { 0 }).collect();

    // Call FMU function
    let status = (fmu.fmi3_set_clock)(
        instance,
        value_references.as_ptr(),
        input.n_value_references as usize,
        clock_values.as_ptr(),
    );

    // Create output
    let output = proto::Fmi3StatusMessage {
        status: status_to_proto(status) as i32,
    };

    serialize_and_reply(&query, &output)?;

    info!("fmi3SetClock completed with status: {:?}", status);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // =============================================================================
    // Mock FMU Library for Testing
    // =============================================================================

    /// Creates a mock FmuLibrary with stub functions for testing
    /// This allows us to test handler logic without loading a real FMU
    fn create_mock_fmu() -> FmuLibrary {
        // Mock function implementations that return predictable values
        extern "C" fn mock_get_version() -> *const i8 {
            b"3.0\0".as_ptr() as *const i8
        }

        extern "C" fn mock_set_debug_logging(
            _instance: *mut std::ffi::c_void,
            _logging_on: i32,
            _n_categories: usize,
            _categories: *const *const i8,
        ) -> fmi3Status {
            fmi3Status::fmi3OK
        }

        extern "C" fn mock_instantiate_co_simulation(
            _instance_name: *const i8,
            _instantiation_token: *const i8,
            _resource_path: *const i8,
            _visible: i32,
            _logging_on: i32,
            _event_mode_used: i32,
            _early_return_allowed: i32,
            _required_intermediate_variables: *const u32,
            _n_required_intermediate_variables: usize,
            _instance_environment: *mut std::ffi::c_void,
            _log_message: Option<extern "C" fn(*mut std::ffi::c_void, fmi3Status, *const i8, *const i8)>,
            _intermediate_update: Option<extern "C" fn(*mut std::ffi::c_void, f64, i32, i32, i32, i32, i32, i32, *mut i32, *mut f64)>,
        ) -> *mut std::ffi::c_void {
            // Return a non-null dummy pointer
            0x1234 as *mut std::ffi::c_void
        }

        extern "C" fn mock_instantiate_model_exchange(
            _instance_name: *const i8,
            _instantiation_token: *const i8,
            _resource_path: *const i8,
            _visible: i32,
            _logging_on: i32,
            _instance_environment: *mut std::ffi::c_void,
            _log_message: Option<extern "C" fn(*mut std::ffi::c_void, fmi3Status, *const i8, *const i8)>,
        ) -> *mut std::ffi::c_void {
            0x5678 as *mut std::ffi::c_void
        }

        extern "C" fn mock_instantiate_scheduled_execution(
            _instance_name: *const i8,
            _instantiation_token: *const i8,
            _resource_path: *const i8,
            _visible: i32,
            _logging_on: i32,
            _instance_environment: *mut std::ffi::c_void,
            _log_message: Option<extern "C" fn(*mut std::ffi::c_void, fmi3Status, *const i8, *const i8)>,
            _clock_update: Option<extern "C" fn(*mut std::ffi::c_void)>,
            _lock_preemption: Option<extern "C" fn()>,
            _unlock_preemption: Option<extern "C" fn()>,
        ) -> *mut std::ffi::c_void {
            0x9ABC as *mut std::ffi::c_void
        }

        extern "C" fn mock_enter_initialization_mode(
            _instance: *mut std::ffi::c_void,
            _tolerance_defined: i32,
            _tolerance: f64,
            _start_time: f64,
            _stop_time_defined: i32,
            _stop_time: f64,
        ) -> fmi3Status {
            fmi3Status::fmi3OK
        }

        extern "C" fn mock_exit_initialization_mode(
            _instance: *mut std::ffi::c_void,
        ) -> fmi3Status {
            fmi3Status::fmi3OK
        }

        extern "C" fn mock_enter_event_mode(
            _instance: *mut std::ffi::c_void,
        ) -> fmi3Status {
            fmi3Status::fmi3OK
        }

        extern "C" fn mock_free_instance(_instance: *mut std::ffi::c_void) {
            // No-op for mock
        }

        extern "C" fn mock_terminate(_instance: *mut std::ffi::c_void) -> fmi3Status {
            fmi3Status::fmi3OK
        }

        extern "C" fn mock_reset(_instance: *mut std::ffi::c_void) -> fmi3Status {
            fmi3Status::fmi3OK
        }

        extern "C" fn mock_do_step(
            _instance: *mut std::ffi::c_void,
            _current_communication_point: f64,
            _communication_step_size: f64,
            _no_set_fmu_state_prior: i32,
            event_handling_needed: *mut i32,
            terminate_simulation: *mut i32,
            early_return: *mut i32,
            last_successful_time: *mut f64,
        ) -> fmi3Status {
            // Set output values
            unsafe {
                if !event_handling_needed.is_null() {
                    *event_handling_needed = 0;
                }
                if !terminate_simulation.is_null() {
                    *terminate_simulation = 0;
                }
                if !early_return.is_null() {
                    *early_return = 0;
                }
                if !last_successful_time.is_null() {
                    *last_successful_time = 1.0;
                }
            }
            fmi3Status::fmi3OK
        }

        extern "C" fn mock_get_float64(
            _instance: *mut std::ffi::c_void,
            _value_references: *const u32,
            n_values: usize,
            values: *mut f64,
            _n_values_out: usize,
        ) -> fmi3Status {
            // Fill values with test data
            unsafe {
                for i in 0..n_values {
                    *values.add(i) = (i as f64) * 1.5;
                }
            }
            fmi3Status::fmi3OK
        }

        extern "C" fn mock_set_float64(
            _instance: *mut std::ffi::c_void,
            _value_references: *const u32,
            _n_value_references: usize,
            _values: *const f64,
            _n_values: usize,
        ) -> fmi3Status {
            fmi3Status::fmi3OK
        }

        extern "C" fn mock_get_int32(
            _instance: *mut std::ffi::c_void,
            _value_references: *const u32,
            n_values: usize,
            values: *mut i32,
            _n_values_out: usize,
        ) -> fmi3Status {
            unsafe {
                for i in 0..n_values {
                    *values.add(i) = (i as i32) * 10;
                }
            }
            fmi3Status::fmi3OK
        }

        extern "C" fn mock_set_int32(
            _instance: *mut std::ffi::c_void,
            _value_references: *const u32,
            _n_value_references: usize,
            _values: *const i32,
            _n_values: usize,
        ) -> fmi3Status {
            fmi3Status::fmi3OK
        }

        extern "C" fn mock_get_boolean(
            _instance: *mut std::ffi::c_void,
            _value_references: *const u32,
            n_values: usize,
            values: *mut i32,
            _n_values_out: usize,
        ) -> fmi3Status {
            unsafe {
                for i in 0..n_values {
                    *values.add(i) = if i % 2 == 0 { 1 } else { 0 };
                }
            }
            fmi3Status::fmi3OK
        }

        extern "C" fn mock_set_boolean(
            _instance: *mut std::ffi::c_void,
            _value_references: *const u32,
            _n_value_references: usize,
            _values: *const i32,
            _n_values: usize,
        ) -> fmi3Status {
            fmi3Status::fmi3OK
        }

        extern "C" fn mock_get_string(
            _instance: *mut std::ffi::c_void,
            _value_references: *const u32,
            n_values: usize,
            values: *mut *const i8,
            _n_values_out: usize,
        ) -> fmi3Status {
            // We need static strings that won't be deallocated
            static TEST_STRING: &[u8] = b"test_value\0";
            unsafe {
                for i in 0..n_values {
                    *values.add(i) = TEST_STRING.as_ptr() as *const i8;
                }
            }
            fmi3Status::fmi3OK
        }

        extern "C" fn mock_set_string(
            _instance: *mut std::ffi::c_void,
            _value_references: *const u32,
            _n_value_references: usize,
            _values: *const *const i8,
            _n_values: usize,
        ) -> fmi3Status {
            fmi3Status::fmi3OK
        }

        extern "C" fn mock_get_clock(
            _instance: *mut std::ffi::c_void,
            _value_references: *const u32,
            n_values: usize,
            values: *mut i32,
        ) -> fmi3Status {
            unsafe {
                for i in 0..n_values {
                    *values.add(i) = if i % 2 == 0 { 1 } else { 0 };
                }
            }
            fmi3Status::fmi3OK
        }

        extern "C" fn mock_set_clock(
            _instance: *mut std::ffi::c_void,
            _value_references: *const u32,
            _n_value_references: usize,
            _values: *const i32,
        ) -> fmi3Status {
            fmi3Status::fmi3OK
        }

        extern "C" fn mock_get_binary(
            _instance: *mut std::ffi::c_void,
            _value_references: *const u32,
            _n_value_references: usize,
            _value_sizes: *mut usize,
            _values: *mut *const u8,
            _n_values: usize,
        ) -> fmi3Status {
            fmi3Status::fmi3OK
        }

        extern "C" fn mock_set_binary(
            _instance: *mut std::ffi::c_void,
            _value_references: *const u32,
            _n_value_references: usize,
            _value_sizes: *const usize,
            _values: *const *const u8,
            _n_values: usize,
        ) -> fmi3Status {
            fmi3Status::fmi3OK
        }

        // Create a stub FmuLibrary with all required function pointers
        // Note: We're using mem::transmute to convert our mock functions to the right type
        // This is safe in tests because we control both sides
        unsafe {
            use std::mem;
            // Create a dummy library handle - we use libloading's unsafe_library to create a fake library
            // This is safe in tests because we never actually call through the library handle
            let library = mem::transmute::<usize, libloading::Library>(0xDEADBEEF);

            let fmu_lib = FmuLibrary {
                library,
                fmi3_get_version: mock_get_version,
                fmi3_set_debug_logging: mock_set_debug_logging,
                fmi3_instantiate_co_simulation: mock_instantiate_co_simulation,
                fmi3_instantiate_model_exchange: mock_instantiate_model_exchange,
                fmi3_instantiate_scheduled_execution: mock_instantiate_scheduled_execution,
                fmi3_enter_initialization_mode: mock_enter_initialization_mode,
                fmi3_exit_initialization_mode: mock_exit_initialization_mode,
                fmi3_enter_event_mode: mock_enter_event_mode,
                fmi3_free_instance: mock_free_instance,
                fmi3_terminate: mock_terminate,
                fmi3_reset: mock_reset,
                fmi3_do_step: mock_do_step,
                fmi3_get_float32: mem::transmute(mock_get_float64 as *const ()),
                fmi3_set_float32: mem::transmute(mock_set_float64 as *const ()),
                fmi3_get_float64: mock_get_float64,
                fmi3_set_float64: mock_set_float64,
                fmi3_get_int8: mem::transmute(mock_get_int32 as *const ()),
                fmi3_set_int8: mem::transmute(mock_set_int32 as *const ()),
                fmi3_get_int16: mem::transmute(mock_get_int32 as *const ()),
                fmi3_set_int16: mem::transmute(mock_set_int32 as *const ()),
                fmi3_get_int32: mock_get_int32,
                fmi3_set_int32: mock_set_int32,
                fmi3_get_int64: mem::transmute(mock_get_int32 as *const ()),
                fmi3_set_int64: mem::transmute(mock_set_int32 as *const ()),
                fmi3_get_uint8: mem::transmute(mock_get_int32 as *const ()),
                fmi3_set_uint8: mem::transmute(mock_set_int32 as *const ()),
                fmi3_get_uint16: mem::transmute(mock_get_int32 as *const ()),
                fmi3_set_uint16: mem::transmute(mock_set_int32 as *const ()),
                fmi3_get_uint32: mem::transmute(mock_get_int32 as *const ()),
                fmi3_set_uint32: mem::transmute(mock_set_int32 as *const ()),
                fmi3_get_uint64: mem::transmute(mock_get_int32 as *const ()),
                fmi3_set_uint64: mem::transmute(mock_set_int32 as *const ()),
                fmi3_get_boolean: mock_get_boolean,
                fmi3_set_boolean: mock_set_boolean,
                fmi3_get_string: mock_get_string,
                fmi3_set_string: mock_set_string,
                fmi3_get_binary: mock_get_binary,
                fmi3_set_binary: mock_set_binary,
                fmi3_get_clock: mock_get_clock,
                fmi3_set_clock: mock_set_clock,
            };

            fmu_lib
        }
    }

    // =============================================================================
    // Helper Functions for Creating Test Messages
    // =============================================================================

    /// Serialize a protobuf message to bytes
    fn serialize_message<T: Message>(msg: &T) -> Vec<u8> {
        let mut buf = Vec::with_capacity(msg.encoded_len());
        msg.encode(&mut buf).unwrap();
        buf
    }

    /// Deserialize a protobuf message from bytes
    fn deserialize_message<T: Message + Default>(bytes: &[u8]) -> T {
        T::decode(bytes).unwrap()
    }

    // =============================================================================
    // Tests for Status Conversion
    // =============================================================================

    #[test]
    fn test_status_conversion() {
        assert_eq!(status_to_proto(fmi3Status::fmi3OK), proto::Status::Ok);
        assert_eq!(status_to_proto(fmi3Status::fmi3Warning), proto::Status::Warning);
        assert_eq!(status_to_proto(fmi3Status::fmi3Discard), proto::Status::Discard);
        assert_eq!(status_to_proto(fmi3Status::fmi3Error), proto::Status::Error);
        assert_eq!(status_to_proto(fmi3Status::fmi3Fatal), proto::Status::Fatal);
    }

    // =============================================================================
    // Tests for Protobuf Message Serialization/Deserialization
    // =============================================================================

    #[test]
    fn test_fmi3_instance_message_serialization() {
        let msg = proto::Fmi3InstanceMessage {
            instance_index: 42,
        };

        let bytes = serialize_message(&msg);
        let decoded: proto::Fmi3InstanceMessage = deserialize_message(&bytes);

        assert_eq!(decoded.instance_index, 42);
    }

    #[test]
    fn test_fmi3_status_message_serialization() {
        let msg = proto::Fmi3StatusMessage {
            status: proto::Status::Ok as i32,
        };

        let bytes = serialize_message(&msg);
        let decoded: proto::Fmi3StatusMessage = deserialize_message(&bytes);

        assert_eq!(decoded.status, proto::Status::Ok as i32);
    }

    #[test]
    fn test_set_debug_logging_message_serialization() {
        let msg = proto::Fmi3SetDebugLoggingMessage {
            instance_index: 0,
            logging_on: true,
            n_categories: 2,
            categories: vec!["logAll".to_string(), "logError".to_string()],
        };

        let bytes = serialize_message(&msg);
        let decoded: proto::Fmi3SetDebugLoggingMessage = deserialize_message(&bytes);

        assert_eq!(decoded.instance_index, 0);
        assert_eq!(decoded.logging_on, true);
        assert_eq!(decoded.n_categories, 2);
        assert_eq!(decoded.categories, vec!["logAll", "logError"]);
    }

    #[test]
    fn test_instantiate_co_simulation_message_serialization() {
        let msg = proto::Fmi3InstantiateCoSimulationMessage {
            instance_name: "TestInstance".to_string(),
            instantiation_token: "{12345678-1234-5678-1234-567812345678}".to_string(),
            resource_path: "/path/to/resources".to_string(),
            visible: false,
            logging_on: true,
            event_mode_used: false,
            early_return_allowed: false,
            required_intermediate_variables: vec![1, 2, 3],
            n_required_intermediate_variables: 3,
        };

        let bytes = serialize_message(&msg);
        let decoded: proto::Fmi3InstantiateCoSimulationMessage = deserialize_message(&bytes);

        assert_eq!(decoded.instance_name, "TestInstance");
        assert_eq!(decoded.instantiation_token, "{12345678-1234-5678-1234-567812345678}");
        assert_eq!(decoded.visible, false);
        assert_eq!(decoded.logging_on, true);
        assert_eq!(decoded.required_intermediate_variables, vec![1, 2, 3]);
    }

    #[test]
    fn test_enter_initialization_mode_message_serialization() {
        let msg = proto::Fmi3EnterInitializationModeMessage {
            instance_index: 0,
            tolerance_defined: true,
            tolerance: 1e-6,
            start_time: 0.0,
            stop_time_defined: true,
            stop_time: 10.0,
        };

        let bytes = serialize_message(&msg);
        let decoded: proto::Fmi3EnterInitializationModeMessage = deserialize_message(&bytes);

        assert_eq!(decoded.instance_index, 0);
        assert_eq!(decoded.tolerance_defined, true);
        assert_eq!(decoded.tolerance, 1e-6);
        assert_eq!(decoded.start_time, 0.0);
        assert_eq!(decoded.stop_time_defined, true);
        assert_eq!(decoded.stop_time, 10.0);
    }

    #[test]
    fn test_do_step_message_serialization() {
        let msg = proto::Fmi3DoStepMessage {
            instance_index: 0,
            current_communication_point: 0.0,
            communication_step_size: 0.1,
            no_set_fmu_state_prior_to_current_point: true,
            event_handling_needed: false,
            terminate_simulation: false,
            early_return: false,
            last_successful_time: 0.0,
        };

        let bytes = serialize_message(&msg);
        let decoded: proto::Fmi3DoStepMessage = deserialize_message(&bytes);

        assert_eq!(decoded.instance_index, 0);
        assert_eq!(decoded.current_communication_point, 0.0);
        assert_eq!(decoded.communication_step_size, 0.1);
        assert_eq!(decoded.no_set_fmu_state_prior_to_current_point, true);
    }

    #[test]
    fn test_get_float64_message_serialization() {
        let msg = proto::Fmi3GetFloat64InputMessage {
            instance_index: 0,
            value_references: vec![1, 2, 3],
            n_value_references: 3,
        };

        let bytes = serialize_message(&msg);
        let decoded: proto::Fmi3GetFloat64InputMessage = deserialize_message(&bytes);

        assert_eq!(decoded.instance_index, 0);
        assert_eq!(decoded.value_references, vec![1, 2, 3]);
        assert_eq!(decoded.n_value_references, 3);
    }

    #[test]
    fn test_set_float64_message_serialization() {
        let msg = proto::Fmi3SetFloat64InputMessage {
            instance_index: 0,
            value_references: vec![1, 2],
            n_value_references: 2,
            values: vec![1.5, 2.5],
            n_values: 2,
        };

        let bytes = serialize_message(&msg);
        let decoded: proto::Fmi3SetFloat64InputMessage = deserialize_message(&bytes);

        assert_eq!(decoded.instance_index, 0);
        assert_eq!(decoded.value_references, vec![1, 2]);
        assert_eq!(decoded.values, vec![1.5, 2.5]);
    }

    // =============================================================================
    // Tests for Type Conversions
    // =============================================================================

    #[test]
    fn test_bool_to_fmi_conversion() {
        // Test boolean to FMI i32 conversion (used in handle_set_boolean)
        let bool_values = vec![true, false, true];
        let fmi_values: Vec<i32> = bool_values.iter().map(|&b| if b { 1 } else { 0 }).collect();

        assert_eq!(fmi_values, vec![1, 0, 1]);
    }

    #[test]
    fn test_fmi_to_bool_conversion() {
        // Test FMI i32 to boolean conversion (used in handle_get_boolean)
        let fmi_values = vec![1, 0, 42, -1];
        let bool_values: Vec<bool> = fmi_values.iter().map(|&v| v != 0).collect();

        assert_eq!(bool_values, vec![true, false, true, true]);
    }

    #[test]
    fn test_value_reference_conversion() {
        // Test proto i32 to FMI u32 value reference conversion
        let proto_refs: Vec<i32> = vec![0, 1, 100, 65535];
        let fmi_refs: Vec<u32> = proto_refs.iter().map(|&vr| vr as u32).collect();

        assert_eq!(fmi_refs, vec![0u32, 1u32, 100u32, 65535u32]);
    }

    #[test]
    fn test_string_to_cstring_conversion() {
        // Test Rust String to CString conversion (used in multiple handlers)
        let rust_strings = vec!["test1".to_string(), "test2".to_string()];
        let c_strings: Vec<CString> = rust_strings
            .iter()
            .map(|s| CString::new(s.as_str()).unwrap())
            .collect();

        assert_eq!(c_strings.len(), 2);
        assert_eq!(c_strings[0].to_str().unwrap(), "test1");
        assert_eq!(c_strings[1].to_str().unwrap(), "test2");
    }

    #[test]
    fn test_string_with_null_byte_handling() {
        // Test that strings with null bytes are handled correctly
        let string_with_null = "test\0embedded";
        let result = CString::new(string_with_null);

        // Should fail because of embedded null
        assert!(result.is_err());

        // Handler should use unwrap_or_default to handle this
        let safe_conversion = CString::new(string_with_null).unwrap_or_default();
        assert_eq!(safe_conversion.to_str().unwrap(), "");
    }

    // =============================================================================
    // Tests for Instance Manager Integration
    // =============================================================================

    // These tests are ignored because they require a valid FMU library handle which
    // cannot be properly mocked without segfaulting.
    #[test]
    #[ignore]
    fn test_instance_manager_with_mock_instances() {
        let manager = InstanceManager::new();
        let _mock_fmu = create_mock_fmu();

        // Create some mock instances
        let ptr1 = 0x1000 as *mut std::ffi::c_void;
        let ptr2 = 0x2000 as *mut std::ffi::c_void;

        let idx1 = manager.add_instance(ptr1);
        let idx2 = manager.add_instance(ptr2);

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);

        // Verify we can retrieve them
        assert_eq!(manager.get_instance(idx1).unwrap(), ptr1);
        assert_eq!(manager.get_instance(idx2).unwrap(), ptr2);

        // Test error case
        assert!(manager.get_instance(999).is_err());
    }

    #[test]
    fn test_instance_manager_remove() {
        let manager = InstanceManager::new();
        let ptr = 0x1000 as *mut std::ffi::c_void;

        let idx = manager.add_instance(ptr);
        assert!(manager.contains_instance(idx));

        // Remove instance
        assert!(manager.remove_instance(idx).is_ok());
        assert!(!manager.contains_instance(idx));

        // Try to remove again - should fail
        assert!(manager.remove_instance(idx).is_err());
    }

    // =============================================================================
    // Tests for Error Handling
    // =============================================================================

    #[test]
    fn test_invalid_instance_index() {
        let manager = InstanceManager::new();

        // Test negative index
        let result = manager.get_instance(-1);
        assert!(result.is_err());

        // Test non-existent index
        let result = manager.get_instance(999);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_categories_array() {
        // Test that empty category arrays are handled correctly
        let categories: Vec<String> = vec![];
        let c_categories: Vec<CString> = categories
            .iter()
            .map(|s| CString::new(s.as_str()))
            .collect::<std::result::Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(c_categories.len(), 0);

        let category_ptrs: Vec<*const i8> = c_categories
            .iter()
            .map(|cs| cs.as_ptr())
            .collect();

        assert_eq!(category_ptrs.len(), 0);
    }

    #[test]
    fn test_empty_value_references() {
        // Test that empty value reference arrays are handled correctly
        let value_refs: Vec<i32> = vec![];
        let fmi_refs: Vec<u32> = value_refs.iter().map(|&vr| vr as u32).collect();

        assert_eq!(fmi_refs.len(), 0);
    }

    // =============================================================================
    // Tests for Binary Data Handling
    // =============================================================================

    #[test]
    fn test_binary_size_array_creation() {
        // Test the pattern used in handle_set_binary for creating size arrays
        let binary_values = vec![
            vec![1u8, 2, 3],
            vec![4, 5],
            vec![6, 7, 8, 9],
        ];

        let mut value_sizes: Vec<usize> = Vec::new();
        let mut binary_buffer: Vec<u8> = Vec::new();

        for binary_value in binary_values.iter() {
            value_sizes.push(binary_value.len());
            binary_buffer.extend_from_slice(binary_value);
        }

        assert_eq!(value_sizes, vec![3, 2, 4]);
        assert_eq!(binary_buffer, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_binary_pointer_array_creation() {
        // Test pointer array creation for binary data
        let binary_buffer = vec![1u8, 2, 3, 4, 5, 6, 7, 8, 9];
        let value_sizes = vec![3, 2, 4];

        let mut binary_ptrs: Vec<*const u8> = Vec::new();
        let mut offset = 0;
        for &size in value_sizes.iter() {
            binary_ptrs.push(unsafe { binary_buffer.as_ptr().add(offset) });
            offset += size;
        }

        assert_eq!(binary_ptrs.len(), 3);

        // Verify the pointers point to the right data
        unsafe {
            assert_eq!(*binary_ptrs[0], 1);
            assert_eq!(*binary_ptrs[1], 4);
            assert_eq!(*binary_ptrs[2], 6);
        }
    }

    // =============================================================================
    // Tests for String Handling
    // =============================================================================

    #[test]
    fn test_string_array_conversion() {
        // Test the pattern used in handle_set_string
        let input_strings = vec![
            "hello".to_string(),
            "world".to_string(),
            "test".to_string(),
        ];

        let c_strings: Vec<CString> = input_strings
            .iter()
            .map(|s| CString::new(s.as_str()).unwrap_or_default())
            .collect();

        let c_string_ptrs: Vec<*const i8> = c_strings
            .iter()
            .map(|cs| cs.as_ptr())
            .collect();

        assert_eq!(c_string_ptrs.len(), 3);

        // Verify the C strings are correct
        unsafe {
            use std::ffi::CStr;
            let str0 = CStr::from_ptr(c_string_ptrs[0]).to_str().unwrap();
            let str1 = CStr::from_ptr(c_string_ptrs[1]).to_str().unwrap();
            let str2 = CStr::from_ptr(c_string_ptrs[2]).to_str().unwrap();

            assert_eq!(str0, "hello");
            assert_eq!(str1, "world");
            assert_eq!(str2, "test");
        }
    }

    #[test]
    fn test_null_string_handling() {
        // Test pattern from handle_get_string for null string pointers
        let c_str_ptr: *const i8 = std::ptr::null();

        let rust_string = if c_str_ptr.is_null() {
            String::new()
        } else {
            unsafe {
                std::ffi::CStr::from_ptr(c_str_ptr)
                    .to_string_lossy()
                    .into_owned()
            }
        };

        assert_eq!(rust_string, "");
    }

    // =============================================================================
    // Tests for Macro-Generated Functions
    // =============================================================================

    #[test]
    fn test_get_value_handler_type_conversions() {
        // Test the type conversion pattern used in impl_get_value_handler macro

        // Float types - no conversion needed
        let float_values: Vec<f64> = vec![1.5, 2.5, 3.5];
        let proto_values: Vec<f64> = float_values.iter().map(|&v| v as f64).collect();
        assert_eq!(proto_values, vec![1.5, 2.5, 3.5]);

        // Integer types - conversion to i32 for proto
        let int8_values: Vec<i8> = vec![1, 2, 3];
        let proto_values: Vec<i32> = int8_values.iter().map(|&v| v as i32).collect();
        assert_eq!(proto_values, vec![1, 2, 3]);

        // Unsigned types - conversion to u32 for proto
        let uint8_values: Vec<u8> = vec![10, 20, 30];
        let proto_values: Vec<u32> = uint8_values.iter().map(|&v| v as u32).collect();
        assert_eq!(proto_values, vec![10, 20, 30]);
    }

    #[test]
    fn test_set_value_handler_type_conversions() {
        // Test the type conversion pattern used in impl_set_value_handler macro

        // Proto to FMI float conversion
        let proto_values: Vec<f64> = vec![1.5, 2.5];
        let fmi_values: Vec<f64> = proto_values.iter().map(|&v| v as f64).collect();
        assert_eq!(fmi_values, vec![1.5, 2.5]);

        // Proto to FMI integer conversion
        let proto_values: Vec<i32> = vec![10, 20];
        let fmi_values: Vec<i16> = proto_values.iter().map(|&v| v as i16).collect();
        assert_eq!(fmi_values, vec![10i16, 20i16]);

        // Proto to FMI unsigned conversion
        let proto_values: Vec<u32> = vec![100, 200];
        let fmi_values: Vec<u8> = proto_values.iter().map(|&v| v as u8).collect();
        assert_eq!(fmi_values, vec![100u8, 200u8]);
    }

    // =============================================================================
    // Integration Tests for Handler Logic (without Zenoh)
    // =============================================================================

    // These tests are ignored because they require a valid FMU library handle which
    // cannot be properly mocked without segfaulting.
    #[test]
    #[ignore]
    fn test_mock_instantiate_returns_valid_pointer() {
        let mock_fmu = create_mock_fmu();

        // Test co-simulation instantiation
        let instance = (mock_fmu.fmi3_instantiate_co_simulation)(
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            0, 0, 0, 0,
            std::ptr::null(),
            0,
            std::ptr::null_mut(),
            None,
            None,
        );

        assert!(!instance.is_null());
        assert_eq!(instance as usize, 0x1234);
    }

    // These tests are ignored because they require a valid FMU library handle which
    // cannot be properly mocked without segfaulting.
    #[test]
    #[ignore]
    fn test_mock_set_debug_logging_returns_ok() {
        let mock_fmu = create_mock_fmu();
        let instance = 0x1234 as *mut std::ffi::c_void;

        let status = (mock_fmu.fmi3_set_debug_logging)(
            instance,
            1,
            0,
            std::ptr::null(),
        );

        assert_eq!(status, fmi3Status::fmi3OK);
    }

    // These tests are ignored because they require a valid FMU library handle which
    // cannot be properly mocked without segfaulting.
    #[test]
    #[ignore]
    fn test_mock_get_float64_fills_array() {
        let mock_fmu = create_mock_fmu();
        let instance = 0x1234 as *mut std::ffi::c_void;

        let value_refs = vec![0u32, 1u32, 2u32];
        let mut values = vec![0.0f64; 3];

        let status = (mock_fmu.fmi3_get_float64)(
            instance,
            value_refs.as_ptr(),
            3,
            values.as_mut_ptr(),
            3,
        );

        assert_eq!(status, fmi3Status::fmi3OK);
        assert_eq!(values, vec![0.0, 1.5, 3.0]);
    }

    // These tests are ignored because they require a valid FMU library handle which
    // cannot be properly mocked without segfaulting.
    #[test]
    #[ignore]
    fn test_mock_get_boolean_fills_array() {
        let mock_fmu = create_mock_fmu();
        let instance = 0x1234 as *mut std::ffi::c_void;

        let value_refs = vec![0u32, 1u32, 2u32, 3u32];
        let mut values = vec![0i32; 4];

        let status = (mock_fmu.fmi3_get_boolean)(
            instance,
            value_refs.as_ptr(),
            4,
            values.as_mut_ptr(),
            4,
        );

        assert_eq!(status, fmi3Status::fmi3OK);
        // Even indices should be 1, odd indices should be 0
        assert_eq!(values, vec![1, 0, 1, 0]);
    }

    // These tests are ignored because they require a valid FMU library handle which
    // cannot be properly mocked without segfaulting.
    #[test]
    #[ignore]
    fn test_mock_do_step_sets_output_parameters() {
        let mock_fmu = create_mock_fmu();
        let instance = 0x1234 as *mut std::ffi::c_void;

        let mut event_handling_needed = -1;
        let mut terminate_simulation = -1;
        let mut early_return = -1;
        let mut last_successful_time = -1.0;

        let status = (mock_fmu.fmi3_do_step)(
            instance,
            0.0,
            0.1,
            0,
            &mut event_handling_needed,
            &mut terminate_simulation,
            &mut early_return,
            &mut last_successful_time,
        );

        assert_eq!(status, fmi3Status::fmi3OK);
        assert_eq!(event_handling_needed, 0);
        assert_eq!(terminate_simulation, 0);
        assert_eq!(early_return, 0);
        assert_eq!(last_successful_time, 1.0);
    }

    // =============================================================================
    // Tests for Edge Cases
    // =============================================================================

    #[test]
    fn test_large_value_reference_array() {
        // Test handling of large arrays
        let large_array: Vec<i32> = (0..1000).collect();
        let fmi_refs: Vec<u32> = large_array.iter().map(|&vr| vr as u32).collect();

        assert_eq!(fmi_refs.len(), 1000);
        assert_eq!(fmi_refs[0], 0);
        assert_eq!(fmi_refs[999], 999);
    }

    #[test]
    fn test_empty_string_array() {
        let empty: Vec<String> = vec![];
        let c_strings: Vec<CString> = empty
            .iter()
            .map(|s| CString::new(s.as_str()).unwrap())
            .collect();

        assert_eq!(c_strings.len(), 0);
    }

    #[test]
    fn test_single_element_arrays() {
        // Test arrays with single elements
        let value_refs = vec![42i32];
        let fmi_refs: Vec<u32> = value_refs.iter().map(|&vr| vr as u32).collect();

        assert_eq!(fmi_refs.len(), 1);
        assert_eq!(fmi_refs[0], 42);
    }

    // =============================================================================
    // Tests for Thread Safety (Instance Manager)
    // =============================================================================

    #[test]
    fn test_concurrent_instance_operations() {
        use std::thread;

        let manager = Arc::new(InstanceManager::new());
        let mut handles = vec![];

        // Spawn multiple threads that add instances
        for i in 0..10 {
            let mgr = Arc::clone(&manager);
            let handle = thread::spawn(move || {
                let ptr = (i * 1000) as *mut std::ffi::c_void;
                mgr.add_instance(ptr)
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Should have 10 instances
        assert_eq!(manager.instance_count(), 10);
    }

    #[test]
    fn test_concurrent_get_operations() {
        use std::thread;

        let manager = Arc::new(InstanceManager::new());

        // Add some instances
        let ptr1 = 0x1000 as *mut std::ffi::c_void;
        let ptr2 = 0x2000 as *mut std::ffi::c_void;
        let idx1 = manager.add_instance(ptr1);
        let idx2 = manager.add_instance(ptr2);

        // Spawn threads that read instances
        let mgr1 = Arc::clone(&manager);
        let handle1 = thread::spawn(move || {
            for _ in 0..100 {
                let _ = mgr1.get_instance(idx1);
            }
        });

        let mgr2 = Arc::clone(&manager);
        let handle2 = thread::spawn(move || {
            for _ in 0..100 {
                let _ = mgr2.get_instance(idx2);
            }
        });

        handle1.join().unwrap();
        handle2.join().unwrap();

        // Instances should still be there
        assert_eq!(manager.get_instance(idx1).unwrap(), ptr1);
        assert_eq!(manager.get_instance(idx2).unwrap(), ptr2);
    }
}
