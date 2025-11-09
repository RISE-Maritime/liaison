// FMI 3.0 function definitions and exports
// This module provides the C ABI exports for the FMI functions

use crate::placeholder::Placeholder;
use crate::proto;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::slice;

// FMI 3.0 types (from FMI standard)
pub type fmi3Instance = *mut c_void;
pub type fmi3InstanceEnvironment = *mut c_void;
pub type fmi3ValueReference = u32;
pub type fmi3Float32 = f32;
pub type fmi3Float64 = f64;
pub type fmi3Int8 = i8;
pub type fmi3UInt8 = u8;
pub type fmi3Int16 = i16;
pub type fmi3UInt16 = u16;
pub type fmi3Int32 = i32;
pub type fmi3UInt32 = u32;
pub type fmi3Int64 = i64;
pub type fmi3UInt64 = u64;
pub type fmi3Boolean = i32;
pub type fmi3Char = c_char;
pub type fmi3String = *const fmi3Char;
pub type fmi3Byte = u8;
pub type fmi3Binary = *const fmi3Byte;
pub type fmi3Clock = i32;
pub type fmi3FMUState = *mut c_void;

// FMI 3.0 status codes
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum fmi3Status {
    fmi3OK = 0,
    fmi3Warning = 1,
    fmi3Discard = 2,
    fmi3Error = 3,
    fmi3Fatal = 4,
}

// Callback function types
pub type fmi3LogMessageCallback = Option<
    extern "C" fn(
        instanceEnvironment: fmi3InstanceEnvironment,
        status: fmi3Status,
        category: fmi3String,
        message: fmi3String,
    ),
>;

pub type fmi3IntermediateUpdateCallback = Option<
    extern "C" fn(
        instanceEnvironment: fmi3InstanceEnvironment,
        intermediateUpdateTime: fmi3Float64,
        eventOccurred: fmi3Boolean,
        clocksTicked: fmi3Boolean,
        intermediateVariableSetAllowed: fmi3Boolean,
        intermediateVariableGetAllowed: fmi3Boolean,
        intermediateStepFinished: fmi3Boolean,
        canReturnEarly: fmi3Boolean,
        earlyReturnRequested: *mut fmi3Boolean,
        earlyReturnTime: *mut fmi3Float64,
    ),
>;

pub type fmi3ClockUpdateCallback =
    Option<extern "C" fn(instanceEnvironment: fmi3InstanceEnvironment)>;
pub type fmi3LockPreemptionCallback = Option<extern "C" fn()>;
pub type fmi3UnlockPreemptionCallback = Option<extern "C" fn()>;

//=============================================================================
// Helper Macros for Instance Casting
//=============================================================================

/// Helper macro to safely cast fmi3Instance to Placeholder reference
/// Returns fmi3Error if instance is null
macro_rules! get_placeholder {
    ($instance:expr) => {{
        if $instance.is_null() {
            return fmi3Status::fmi3Error;
        }
        unsafe { &*(($instance) as *const Placeholder) }
    }};
}

/// Helper macro for void functions that need to check null instance
macro_rules! check_instance_or_return {
    ($instance:expr) => {{
        if $instance.is_null() {
            return;
        }
        unsafe { &*(($instance) as *const Placeholder) }
    }};
}

/// Macro to safely cast instance to Placeholder (alias for get_placeholder)
macro_rules! cast_instance {
    ($instance:expr) => {{
        if $instance.is_null() {
            return fmi3Status::fmi3Error;
        }
        unsafe { &*($instance as *const Placeholder) }
    }};
}

//=============================================================================
// Helper Functions
//=============================================================================

// Helper function to safely convert C string to Rust String
unsafe fn c_str_to_string(c_str: fmi3String) -> String {
    if c_str.is_null() {
        String::new()
    } else {
        CStr::from_ptr(c_str).to_string_lossy().into_owned()
    }
}

// Helper function to log fatal error and return null
unsafe fn log_fatal_and_return_null(
    instance_environment: fmi3InstanceEnvironment,
    log_message: fmi3LogMessageCallback,
    error_msg: &str,
) -> fmi3Instance {
    if let (Some(callback), Ok(category), Ok(message)) = (
        log_message,
        std::ffi::CString::new("Zenoh"),
        std::ffi::CString::new(error_msg),
    ) {
        callback(
            instance_environment,
            fmi3Status::fmi3Fatal,
            category.as_ptr(),
            message.as_ptr(),
        );
    }
    std::ptr::null_mut()
}

//=============================================================================
// FMI 3.0 Version and Common Functions
//=============================================================================

#[no_mangle]
pub extern "C" fn fmi3GetVersion() -> fmi3String {
    c"3.0".as_ptr() as fmi3String
}

/// Configure debug logging via Zenoh
#[no_mangle]
pub extern "C" fn fmi3SetDebugLogging(
    instance: fmi3Instance,
    logging_on: fmi3Boolean,
    n_categories: usize,
    categories: *const fmi3String,
) -> fmi3Status {
    // Get placeholder instance (returns fmi3Error if null)
    let placeholder = get_placeholder!(instance);

    // Parse categories from C strings if provided
    let categories_vec = if !categories.is_null() && n_categories > 0 {
        unsafe {
            let categories_slice = slice::from_raw_parts(categories, n_categories);
            categories_slice
                .iter()
                .filter_map(|&category_ptr| {
                    if !category_ptr.is_null() {
                        CStr::from_ptr(category_ptr).to_str().ok().map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect()
        }
    } else {
        Vec::new()
    };

    // Create input message with instance reference
    let input = proto::Fmi3SetDebugLoggingMessage {
        instance_index: placeholder.instance_index,
        logging_on: logging_on != 0,
        n_categories: n_categories as i32,
        categories: categories_vec,
    };

    // Perform Zenoh query
    match placeholder.query::<proto::Fmi3SetDebugLoggingMessage, proto::Fmi3StatusMessage>(
        "fmi3SetDebugLogging",
        &input,
    ) {
        Ok(output) => {
            // Convert protobuf status to fmi3Status
            proto::Status::try_from(output.status)
                .unwrap_or(proto::Status::Error)
                .into()
        }
        Err(_) => fmi3Status::fmi3Fatal,
    }
}

/// Free the FMU instance
#[no_mangle]
pub extern "C" fn fmi3FreeInstance(instance: fmi3Instance) {
    // Check for null instance
    let placeholder = check_instance_or_return!(instance);

    // Create input message to notify server
    let input = proto::Fmi3InstanceMessage {
        instance_index: placeholder.instance_index,
    };

    // Perform Zenoh query to notify server
    // We ignore the result since we're cleaning up anyway
    let _ = placeholder
        .query::<proto::Fmi3InstanceMessage, proto::Fmi3StatusMessage>("fmi3FreeInstance", &input);

    // Clean up the Placeholder
    // SAFETY: The instance was created by Box::into_raw() in the instantiate functions,
    // so it's safe to convert it back to a Box for proper deallocation
    unsafe {
        let _ = Box::from_raw(instance as *mut Placeholder);
    }
    // The Placeholder's Drop implementation will:
    // - Undeclare the log message subscriber
    // - Close the Zenoh session (via Arc drop)
}

//=============================================================================
// FMI 3.0 Instantiation Functions
//=============================================================================

#[no_mangle]
pub extern "C" fn fmi3InstantiateCoSimulation(
    instance_name: fmi3String,
    instantiation_token: fmi3String,
    resource_path: fmi3String,
    visible: fmi3Boolean,
    logging_on: fmi3Boolean,
    event_mode_used: fmi3Boolean,
    early_return_allowed: fmi3Boolean,
    required_intermediate_variables: *const fmi3ValueReference,
    n_required_intermediate_variables: usize,
    instance_environment: fmi3InstanceEnvironment,
    log_message: fmi3LogMessageCallback,
    _intermediate_update: fmi3IntermediateUpdateCallback,
) -> fmi3Instance {
    unsafe {
        // Create Placeholder instance
        let mut placeholder = match Placeholder::new(instance_environment, log_message) {
            Ok(p) => p,
            Err(e) => {
                let error_msg = format!("Failed to create Placeholder: {}", e);
                return log_fatal_and_return_null(instance_environment, log_message, &error_msg);
            }
        };

        // Copy required intermediate variables
        let intermediate_vars = if !required_intermediate_variables.is_null() && n_required_intermediate_variables > 0 {
            let vars_slice = slice::from_raw_parts(
                required_intermediate_variables,
                n_required_intermediate_variables,
            );
            vars_slice.iter().map(|&v| v as i32).collect()
        } else {
            Vec::new()
        };

        // Build protobuf input message
        let input = proto::Fmi3InstantiateCoSimulationMessage {
            instance_name: c_str_to_string(instance_name),
            instantiation_token: c_str_to_string(instantiation_token),
            resource_path: c_str_to_string(resource_path),
            visible: visible != 0,
            logging_on: logging_on != 0,
            event_mode_used: event_mode_used != 0,
            early_return_allowed: early_return_allowed != 0,
            required_intermediate_variables: intermediate_vars,
            n_required_intermediate_variables: n_required_intermediate_variables as i32,
        };

        // Send Zenoh query to server
        let output: proto::Fmi3InstanceMessage = match placeholder
            .query("fmi3InstantiateCoSimulation", &input)
        {
            Ok(o) => o,
            Err(e) => {
                let error_msg = format!("Query failed: {}", e);
                return log_fatal_and_return_null(instance_environment, log_message, &error_msg);
            }
        };

        // Set instance index in Placeholder
        placeholder.set_instance_index(output.instance_index);

        // Return Placeholder as fmi3Instance pointer
        Box::into_raw(Box::new(placeholder)) as fmi3Instance
    }
}

#[no_mangle]
pub extern "C" fn fmi3InstantiateModelExchange(
    instance_name: fmi3String,
    instantiation_token: fmi3String,
    resource_path: fmi3String,
    visible: fmi3Boolean,
    logging_on: fmi3Boolean,
    instance_environment: fmi3InstanceEnvironment,
    log_message: fmi3LogMessageCallback,
) -> fmi3Instance {
    unsafe {
        // Create Placeholder instance
        let mut placeholder = match Placeholder::new(instance_environment, log_message) {
            Ok(p) => p,
            Err(e) => {
                let error_msg = format!("Failed to create Placeholder: {}", e);
                return log_fatal_and_return_null(instance_environment, log_message, &error_msg);
            }
        };

        // Build protobuf input message
        let input = proto::Fmi3InstantiateModelExchangeMessage {
            instance_name: c_str_to_string(instance_name),
            instantiation_token: c_str_to_string(instantiation_token),
            resource_path: c_str_to_string(resource_path),
            visible: visible != 0,
            logging_on: logging_on != 0,
        };

        // Send Zenoh query to server
        let output: proto::Fmi3InstanceMessage = match placeholder
            .query("fmi3InstantiateModelExchange", &input)
        {
            Ok(o) => o,
            Err(e) => {
                let error_msg = format!("Query failed: {}", e);
                return log_fatal_and_return_null(instance_environment, log_message, &error_msg);
            }
        };

        // Set instance index in Placeholder
        placeholder.set_instance_index(output.instance_index);

        // Return Placeholder as fmi3Instance pointer
        Box::into_raw(Box::new(placeholder)) as fmi3Instance
    }
}

#[no_mangle]
pub extern "C" fn fmi3InstantiateScheduledExecution(
    instance_name: fmi3String,
    instantiation_token: fmi3String,
    resource_path: fmi3String,
    visible: fmi3Boolean,
    logging_on: fmi3Boolean,
    instance_environment: fmi3InstanceEnvironment,
    log_message: fmi3LogMessageCallback,
    _clock_update: fmi3ClockUpdateCallback,
    _lock_preemption: fmi3LockPreemptionCallback,
    _unlock_preemption: fmi3UnlockPreemptionCallback,
) -> fmi3Instance {
    unsafe {
        // Create Placeholder instance
        let mut placeholder = match Placeholder::new(instance_environment, log_message) {
            Ok(p) => p,
            Err(e) => {
                let error_msg = format!("Failed to create Placeholder: {}", e);
                return log_fatal_and_return_null(instance_environment, log_message, &error_msg);
            }
        };

        // Build protobuf input message
        let input = proto::Fmi3InstantiateScheduledExecutionMessage {
            instance_name: c_str_to_string(instance_name),
            instantiation_token: c_str_to_string(instantiation_token),
            resource_path: c_str_to_string(resource_path),
            visible: visible != 0,
            logging_on: logging_on != 0,
        };

        // Send Zenoh query to server
        let output: proto::Fmi3InstanceMessage = match placeholder
            .query("fmi3InstantiateScheduledExecution", &input)
        {
            Ok(o) => o,
            Err(e) => {
                let error_msg = format!("Query failed: {}", e);
                return log_fatal_and_return_null(instance_environment, log_message, &error_msg);
            }
        };

        // Set instance index in Placeholder
        placeholder.set_instance_index(output.instance_index);

        // Return Placeholder as fmi3Instance pointer
        Box::into_raw(Box::new(placeholder)) as fmi3Instance
    }
}

//=============================================================================
// FMI 3.0 Lifecycle Functions
//=============================================================================

#[no_mangle]
pub extern "C" fn fmi3EnterInitializationMode(
    instance: fmi3Instance,
    tolerance_defined: fmi3Boolean,
    tolerance: fmi3Float64,
    start_time: fmi3Float64,
    stop_time_defined: fmi3Boolean,
    stop_time: fmi3Float64,
) -> fmi3Status {
    let placeholder = cast_instance!(instance);

    // Create input message
    let input = proto::Fmi3EnterInitializationModeMessage {
        instance_index: placeholder.instance_index,
        tolerance_defined: tolerance_defined != 0,
        tolerance,
        start_time,
        stop_time_defined: stop_time_defined != 0,
        stop_time,
    };

    // Send query and get response
    match placeholder.query::<_, proto::Fmi3StatusMessage>("fmi3EnterInitializationMode", &input) {
        Ok(output) => {
            // Convert protobuf status to FMI status
            proto::Status::try_from(output.status)
                .unwrap_or(proto::Status::Error)
                .into()
        }
        Err(_) => fmi3Status::fmi3Fatal,
    }
}

#[no_mangle]
pub extern "C" fn fmi3ExitInitializationMode(instance: fmi3Instance) -> fmi3Status {
    let placeholder = cast_instance!(instance);

    // Create input message
    let input = proto::Fmi3InstanceMessage {
        instance_index: placeholder.instance_index,
    };

    // Send query and get response
    match placeholder.query::<_, proto::Fmi3StatusMessage>("fmi3ExitInitializationMode", &input) {
        Ok(output) => proto::Status::try_from(output.status)
            .unwrap_or(proto::Status::Error)
            .into(),
        Err(_) => fmi3Status::fmi3Fatal,
    }
}

#[no_mangle]
pub extern "C" fn fmi3Terminate(instance: fmi3Instance) -> fmi3Status {
    let placeholder = cast_instance!(instance);

    // Create input message
    let input = proto::Fmi3InstanceMessage {
        instance_index: placeholder.instance_index,
    };

    // Send query and get response
    match placeholder.query::<_, proto::Fmi3StatusMessage>("fmi3Terminate", &input) {
        Ok(output) => proto::Status::try_from(output.status)
            .unwrap_or(proto::Status::Error)
            .into(),
        Err(_) => fmi3Status::fmi3Fatal,
    }
}

#[no_mangle]
pub extern "C" fn fmi3Reset(instance: fmi3Instance) -> fmi3Status {
    let placeholder = cast_instance!(instance);

    // Create input message
    let input = proto::Fmi3InstanceMessage {
        instance_index: placeholder.instance_index,
    };

    // Send query and get response
    match placeholder.query::<_, proto::Fmi3StatusMessage>("fmi3Reset", &input) {
        Ok(output) => proto::Status::try_from(output.status)
            .unwrap_or(proto::Status::Error)
            .into(),
        Err(_) => fmi3Status::fmi3Fatal,
    }
}

#[no_mangle]
pub extern "C" fn fmi3EnterConfigurationMode(instance: fmi3Instance) -> fmi3Status {
    let placeholder = cast_instance!(instance);

    // Create input message
    let input = proto::Fmi3InstanceMessage {
        instance_index: placeholder.instance_index,
    };

    // Send query and get response
    match placeholder.query::<_, proto::Fmi3StatusMessage>("fmi3EnterConfigurationMode", &input) {
        Ok(output) => proto::Status::try_from(output.status)
            .unwrap_or(proto::Status::Error)
            .into(),
        Err(_) => fmi3Status::fmi3Fatal,
    }
}

#[no_mangle]
pub extern "C" fn fmi3ExitConfigurationMode(instance: fmi3Instance) -> fmi3Status {
    let placeholder = cast_instance!(instance);

    // Create input message
    let input = proto::Fmi3InstanceMessage {
        instance_index: placeholder.instance_index,
    };

    // Send query and get response
    match placeholder.query::<_, proto::Fmi3StatusMessage>("fmi3ExitConfigurationMode", &input) {
        Ok(output) => proto::Status::try_from(output.status)
            .unwrap_or(proto::Status::Error)
            .into(),
        Err(_) => fmi3Status::fmi3Fatal,
    }
}

//=============================================================================
// FMI 3.0 Co-Simulation Function
//=============================================================================

#[no_mangle]
pub extern "C" fn fmi3DoStep(
    instance: fmi3Instance,
    current_communication_point: fmi3Float64,
    communication_step_size: fmi3Float64,
    no_set_fmu_state_prior_to_current_point: fmi3Boolean,
    event_handling_needed: *mut fmi3Boolean,
    terminate_simulation: *mut fmi3Boolean,
    early_return: *mut fmi3Boolean,
    last_successful_time: *mut fmi3Float64,
) -> fmi3Status {
    // Null pointer check for instance
    if instance.is_null() {
        return fmi3Status::fmi3Error;
    }

    // Cast instance to Placeholder
    let placeholder = unsafe { &*(instance as *const Placeholder) };

    // Null pointer checks for output parameters
    if event_handling_needed.is_null()
        || terminate_simulation.is_null()
        || early_return.is_null()
        || last_successful_time.is_null()
    {
        return fmi3Status::fmi3Error;
    }

    // Read current values from output pointers (before the step)
    let event_handling_needed_value = unsafe { *event_handling_needed };
    let terminate_simulation_value = unsafe { *terminate_simulation };
    let early_return_value = unsafe { *early_return };
    let last_successful_time_value = unsafe { *last_successful_time };

    // Build the protobuf input message
    let input = proto::Fmi3DoStepMessage {
        instance_index: placeholder.instance_index,
        current_communication_point,
        communication_step_size,
        no_set_fmu_state_prior_to_current_point: no_set_fmu_state_prior_to_current_point != 0,
        event_handling_needed: event_handling_needed_value != 0,
        terminate_simulation: terminate_simulation_value != 0,
        early_return: early_return_value != 0,
        last_successful_time: last_successful_time_value,
    };

    // Send Zenoh query and get response
    let output: proto::Fmi3StatusMessage = match placeholder.query("fmi3DoStep", &input) {
        Ok(response) => response,
        Err(_) => {
            // Error already logged by placeholder.query()
            return fmi3Status::fmi3Fatal;
        }
    };

    // Note: The current protobuf definition only returns a status message.
    // In a complete implementation, the response should include updated values
    // for eventHandlingNeeded, terminateSimulation, earlyReturn, and lastSuccessfulTime.

    // Convert protobuf status to fmi3Status
    proto::Status::try_from(output.status)
        .unwrap_or(proto::Status::Error)
        .into()
}

//=============================================================================
// FMI 3.0 Get/Set Value Functions - Macros
//=============================================================================

/// Macro to generate FMI3 Get functions for simple numeric types
macro_rules! define_fmi3_get_value_function {
    ($type_name:ident, $fmi3_type:ty, $proto_input:ty, $proto_output:ty, $proto_value_type:ty) => {
        paste::paste! {
            #[no_mangle]
            pub extern "C" fn [<fmi3Get $type_name>](
                instance: fmi3Instance,
                value_references: *const fmi3ValueReference,
                n_value_references: usize,
                values: *mut $fmi3_type,
                n_values: usize,
            ) -> fmi3Status {
                // Null pointer checks
                if instance.is_null() {
                    return fmi3Status::fmi3Error;
                }
                if value_references.is_null() || values.is_null() {
                    return fmi3Status::fmi3Error;
                }
                if n_value_references == 0 || n_values == 0 {
                    return fmi3Status::fmi3Error;
                }

                // Get the Placeholder instance
                let placeholder = unsafe { &*(instance as *const Placeholder) };

                // Build the input message
                let value_refs_slice = unsafe {
                    slice::from_raw_parts(value_references, n_value_references)
                };
                let input = $proto_input {
                    instance_index: placeholder.instance_index,
                    value_references: value_refs_slice.iter().map(|&vr| vr as i32).collect(),
                    n_value_references: n_value_references as i32,
                };

                // Execute the query
                let output: $proto_output = match placeholder.query(
                    concat!("fmi3Get", stringify!($type_name)),
                    &input,
                ) {
                    Ok(out) => out,
                    Err(_) => return fmi3Status::fmi3Fatal,
                };

                // Write values to output array
                let values_slice = unsafe {
                    slice::from_raw_parts_mut(values, n_values)
                };

                let values_count = output.values.len().min(n_values);
                for (idx, &val) in output.values.iter().take(values_count).enumerate() {
                    values_slice[idx] = val as $fmi3_type;
                }

                // Return status
                proto::Status::try_from(output.status)
                    .unwrap_or(proto::Status::Error)
                    .into()
            }
        }
    };
}

/// Macro to generate FMI3 Set functions for simple numeric types
macro_rules! define_fmi3_set_value_function {
    ($type_name:ident, $fmi3_type:ty, $proto_input:ty, $proto_value_type:ty) => {
        paste::paste! {
            #[no_mangle]
            pub extern "C" fn [<fmi3Set $type_name>](
                instance: fmi3Instance,
                value_references: *const fmi3ValueReference,
                n_value_references: usize,
                values: *const $fmi3_type,
                n_values: usize,
            ) -> fmi3Status {
                // Null pointer checks
                if instance.is_null() {
                    return fmi3Status::fmi3Error;
                }
                if value_references.is_null() || values.is_null() {
                    return fmi3Status::fmi3Error;
                }
                if n_value_references == 0 || n_values == 0 {
                    return fmi3Status::fmi3Error;
                }

                // Get the Placeholder instance
                let placeholder = unsafe { &*(instance as *const Placeholder) };

                // Build the input message
                let value_refs_slice = unsafe {
                    slice::from_raw_parts(value_references, n_value_references)
                };
                let values_slice = unsafe {
                    slice::from_raw_parts(values, n_values)
                };
                let input = $proto_input {
                    instance_index: placeholder.instance_index,
                    value_references: value_refs_slice.iter().map(|&vr| vr as i32).collect(),
                    n_value_references: n_value_references as i32,
                    values: values_slice.iter().map(|&val| val as $proto_value_type).collect(),
                    n_values: n_values as i32,
                };

                // Execute the query
                let output: proto::Fmi3StatusMessage = match placeholder.query(
                    concat!("fmi3Set", stringify!($type_name)),
                    &input,
                ) {
                    Ok(out) => out,
                    Err(_) => return fmi3Status::fmi3Fatal,
                };

                // Return status
                proto::Status::try_from(output.status)
                    .unwrap_or(proto::Status::Error)
                    .into()
            }
        }
    };
}

//=============================================================================
// Float64 and Float32 Functions
//=============================================================================

define_fmi3_get_value_function!(
    Float64,
    fmi3Float64,
    proto::Fmi3GetFloat64InputMessage,
    proto::Fmi3GetFloat64OutputMessage,
    f64
);

define_fmi3_set_value_function!(Float64, fmi3Float64, proto::Fmi3SetFloat64InputMessage, f64);

define_fmi3_get_value_function!(
    Float32,
    fmi3Float32,
    proto::Fmi3GetFloat32InputMessage,
    proto::Fmi3GetFloat32OutputMessage,
    f32
);

define_fmi3_set_value_function!(Float32, fmi3Float32, proto::Fmi3SetFloat32InputMessage, f32);

//=============================================================================
// Integer Functions
//=============================================================================

define_fmi3_get_value_function!(
    Int8,
    fmi3Int8,
    proto::Fmi3GetInt8InputMessage,
    proto::Fmi3GetInt8OutputMessage,
    i32
);

define_fmi3_set_value_function!(Int8, fmi3Int8, proto::Fmi3SetInt8InputMessage, i32);

define_fmi3_get_value_function!(
    UInt8,
    fmi3UInt8,
    proto::Fmi3GetUInt8InputMessage,
    proto::Fmi3GetUInt8OutputMessage,
    u32
);

define_fmi3_set_value_function!(UInt8, fmi3UInt8, proto::Fmi3SetUInt8InputMessage, u32);

define_fmi3_get_value_function!(
    Int16,
    fmi3Int16,
    proto::Fmi3GetInt16InputMessage,
    proto::Fmi3GetInt16OutputMessage,
    i32
);

define_fmi3_set_value_function!(Int16, fmi3Int16, proto::Fmi3SetInt16InputMessage, i32);

define_fmi3_get_value_function!(
    UInt16,
    fmi3UInt16,
    proto::Fmi3GetUInt16InputMessage,
    proto::Fmi3GetUInt16OutputMessage,
    u32
);

define_fmi3_set_value_function!(UInt16, fmi3UInt16, proto::Fmi3SetUInt16InputMessage, u32);

define_fmi3_get_value_function!(
    Int32,
    fmi3Int32,
    proto::Fmi3GetInt32InputMessage,
    proto::Fmi3GetInt32OutputMessage,
    i32
);

define_fmi3_set_value_function!(Int32, fmi3Int32, proto::Fmi3SetInt32InputMessage, i32);

define_fmi3_get_value_function!(
    UInt32,
    fmi3UInt32,
    proto::Fmi3GetUInt32InputMessage,
    proto::Fmi3GetUInt32OutputMessage,
    u32
);

define_fmi3_set_value_function!(UInt32, fmi3UInt32, proto::Fmi3SetUInt32InputMessage, u32);

define_fmi3_get_value_function!(
    Int64,
    fmi3Int64,
    proto::Fmi3GetInt64InputMessage,
    proto::Fmi3GetInt64OutputMessage,
    i64
);

define_fmi3_set_value_function!(Int64, fmi3Int64, proto::Fmi3SetInt64InputMessage, i64);

define_fmi3_get_value_function!(
    UInt64,
    fmi3UInt64,
    proto::Fmi3GetUInt64InputMessage,
    proto::Fmi3GetUInt64OutputMessage,
    u64
);

define_fmi3_set_value_function!(UInt64, fmi3UInt64, proto::Fmi3SetUInt64InputMessage, u64);

//=============================================================================
// Boolean Functions
//=============================================================================

// Boolean functions need special handling due to fmi3Boolean being i32
#[no_mangle]
pub extern "C" fn fmi3GetBoolean(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *mut fmi3Boolean,
    n_values: usize,
) -> fmi3Status {
    if instance.is_null() || value_references.is_null() || values.is_null() {
        return fmi3Status::fmi3Error;
    }
    if n_value_references == 0 || n_values == 0 {
        return fmi3Status::fmi3Error;
    }

    let placeholder = unsafe { &*(instance as *const Placeholder) };

    let value_refs_slice = unsafe { slice::from_raw_parts(value_references, n_value_references) };
    let input = proto::Fmi3GetBooleanInputMessage {
        instance_index: placeholder.instance_index,
        value_references: value_refs_slice.iter().map(|&vr| vr as i32).collect(),
        n_value_references: n_value_references as i32,
    };

    let output: proto::Fmi3GetBooleanOutputMessage =
        match placeholder.query("fmi3GetBoolean", &input) {
            Ok(out) => out,
            Err(_) => return fmi3Status::fmi3Fatal,
        };

    let values_slice = unsafe { slice::from_raw_parts_mut(values, n_values) };

    let values_count = output.values.len().min(n_values);
    for (idx, &val) in output.values.iter().take(values_count).enumerate() {
        values_slice[idx] = if val { 1 } else { 0 };
    }

    proto::Status::try_from(output.status)
        .unwrap_or(proto::Status::Error)
        .into()
}

#[no_mangle]
pub extern "C" fn fmi3SetBoolean(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *const fmi3Boolean,
    n_values: usize,
) -> fmi3Status {
    if instance.is_null() || value_references.is_null() || values.is_null() {
        return fmi3Status::fmi3Error;
    }
    if n_value_references == 0 || n_values == 0 {
        return fmi3Status::fmi3Error;
    }

    let placeholder = unsafe { &*(instance as *const Placeholder) };

    let value_refs_slice = unsafe { slice::from_raw_parts(value_references, n_value_references) };
    let values_slice = unsafe { slice::from_raw_parts(values, n_values) };
    let input = proto::Fmi3SetBooleanInputMessage {
        instance_index: placeholder.instance_index,
        value_references: value_refs_slice.iter().map(|&vr| vr as i32).collect(),
        n_value_references: n_value_references as i32,
        values: values_slice.iter().map(|&val| val != 0).collect(),
        n_values: n_values as i32,
    };

    let output: proto::Fmi3StatusMessage = match placeholder.query("fmi3SetBoolean", &input) {
        Ok(out) => out,
        Err(_) => return fmi3Status::fmi3Fatal,
    };

    proto::Status::try_from(output.status)
        .unwrap_or(proto::Status::Error)
        .into()
}

//=============================================================================
// String Functions
//=============================================================================

#[no_mangle]
pub extern "C" fn fmi3GetString(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *mut fmi3String,
    n_values: usize,
) -> fmi3Status {
    // Null pointer checks
    if instance.is_null() || value_references.is_null() || values.is_null() {
        return fmi3Status::fmi3Error;
    }
    if n_value_references == 0 || n_values == 0 {
        return fmi3Status::fmi3Error;
    }

    let placeholder = unsafe { &*(instance as *const Placeholder) };

    let value_refs_slice = unsafe { slice::from_raw_parts(value_references, n_value_references) };
    let input = proto::Fmi3GetStringInputMessage {
        instance_index: placeholder.instance_index,
        value_references: value_refs_slice.iter().map(|&vr| vr as i32).collect(),
        n_value_references: n_value_references as i32,
    };

    let output: proto::Fmi3GetStringOutputMessage = match placeholder.query("fmi3GetString", &input)
    {
        Ok(out) => out,
        Err(_) => return fmi3Status::fmi3Fatal,
    };

    let values_slice = unsafe { slice::from_raw_parts_mut(values, n_values) };

    let values_count = output.values.len().min(n_values);
    for (idx, val) in output.values.iter().take(values_count).enumerate() {
        match CString::new(val.as_str()) {
            Ok(c_str) => {
                values_slice[idx] = c_str.into_raw();
            }
            Err(_) => {
                return fmi3Status::fmi3Error;
            }
        }
    }

    proto::Status::try_from(output.status)
        .unwrap_or(proto::Status::Error)
        .into()
}

#[no_mangle]
pub extern "C" fn fmi3SetString(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *const fmi3String,
    n_values: usize,
) -> fmi3Status {
    // Null pointer checks
    if instance.is_null() || value_references.is_null() || values.is_null() {
        return fmi3Status::fmi3Error;
    }
    if n_value_references == 0 || n_values == 0 {
        return fmi3Status::fmi3Error;
    }

    let placeholder = unsafe { &*(instance as *const Placeholder) };

    let value_refs_slice = unsafe { slice::from_raw_parts(value_references, n_value_references) };
    let values_slice = unsafe { slice::from_raw_parts(values, n_values) };

    // Convert C strings to Rust strings
    let mut string_values = Vec::with_capacity(n_values);
    for &c_str_ptr in values_slice {
        if c_str_ptr.is_null() {
            return fmi3Status::fmi3Error;
        }

        let c_str = unsafe { CStr::from_ptr(c_str_ptr) };
        match c_str.to_str() {
            Ok(rust_str) => {
                string_values.push(rust_str.to_string());
            }
            Err(_) => {
                return fmi3Status::fmi3Error;
            }
        }
    }

    let input = proto::Fmi3SetStringInputMessage {
        instance_index: placeholder.instance_index,
        value_references: value_refs_slice.iter().map(|&vr| vr as i32).collect(),
        n_value_references: n_value_references as i32,
        values: string_values,
        n_values: n_values as i32,
    };

    let output: proto::Fmi3StatusMessage = match placeholder.query("fmi3SetString", &input) {
        Ok(out) => out,
        Err(_) => return fmi3Status::fmi3Fatal,
    };

    proto::Status::try_from(output.status)
        .unwrap_or(proto::Status::Error)
        .into()
}

//=============================================================================
// Binary Functions
//=============================================================================

#[no_mangle]
pub extern "C" fn fmi3GetBinary(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    value_sizes: *mut usize,
    values: *mut fmi3Binary,
    n_values: usize,
) -> fmi3Status {
    // Null pointer checks
    if instance.is_null() || value_references.is_null() || value_sizes.is_null() || values.is_null()
    {
        return fmi3Status::fmi3Error;
    }
    if n_value_references == 0 || n_values == 0 {
        return fmi3Status::fmi3Error;
    }

    let placeholder = unsafe { &*(instance as *const Placeholder) };

    let value_refs_slice = unsafe { slice::from_raw_parts(value_references, n_value_references) };
    let input = proto::Fmi3GetBinaryInputMessage {
        instance_index: placeholder.instance_index,
        value_references: value_refs_slice.iter().map(|&vr| vr as i32).collect(),
        n_value_references: n_value_references as i32,
    };

    let output: proto::Fmi3GetBinaryOutputMessage = match placeholder.query("fmi3GetBinary", &input)
    {
        Ok(out) => out,
        Err(_) => return fmi3Status::fmi3Fatal,
    };

    let sizes_slice = unsafe { slice::from_raw_parts_mut(value_sizes, n_values) };
    let values_slice = unsafe { slice::from_raw_parts_mut(values, n_values) };

    let values_count = output.values.len().min(n_values);

    for (idx, binary_data) in output.values.iter().take(values_count).enumerate() {
        let binary_size = binary_data.len();

        sizes_slice[idx] = binary_size;

        let mut binary_vec = binary_data.clone();
        let binary_ptr = binary_vec.as_mut_ptr();
        std::mem::forget(binary_vec);

        values_slice[idx] = binary_ptr;
    }

    proto::Status::try_from(output.status)
        .unwrap_or(proto::Status::Error)
        .into()
}

#[no_mangle]
pub extern "C" fn fmi3SetBinary(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    value_sizes: *const usize,
    values: *const fmi3Binary,
    n_values: usize,
) -> fmi3Status {
    // Null pointer checks
    if instance.is_null() || value_references.is_null() || value_sizes.is_null() || values.is_null()
    {
        return fmi3Status::fmi3Error;
    }
    if n_value_references == 0 || n_values == 0 {
        return fmi3Status::fmi3Error;
    }

    let placeholder = unsafe { &*(instance as *const Placeholder) };

    let value_refs_slice = unsafe { slice::from_raw_parts(value_references, n_value_references) };
    let sizes_slice = unsafe { slice::from_raw_parts(value_sizes, n_values) };
    let values_slice = unsafe { slice::from_raw_parts(values, n_values) };

    // Convert binary data to Vec<Vec<u8>>
    let mut binary_values = Vec::with_capacity(n_values);
    for (binary_ptr, &binary_size) in values_slice.iter().zip(sizes_slice.iter()) {
        if binary_ptr.is_null() {
            return fmi3Status::fmi3Error;
        }

        let binary_data = unsafe { slice::from_raw_parts(*binary_ptr, binary_size) };
        binary_values.push(binary_data.to_vec());
    }

    let input = proto::Fmi3SetBinaryInputMessage {
        instance_index: placeholder.instance_index,
        value_references: value_refs_slice.iter().map(|&vr| vr as i32).collect(),
        n_value_references: n_value_references as i32,
        values: binary_values,
        n_values: n_values as i32,
    };

    let output: proto::Fmi3StatusMessage = match placeholder.query("fmi3SetBinary", &input) {
        Ok(out) => out,
        Err(_) => return fmi3Status::fmi3Fatal,
    };

    proto::Status::try_from(output.status)
        .unwrap_or(proto::Status::Error)
        .into()
}

//=============================================================================
// Clock Functions
//=============================================================================

#[no_mangle]
pub extern "C" fn fmi3GetClock(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *mut fmi3Clock,
) -> fmi3Status {
    // Null pointer checks
    if instance.is_null() || value_references.is_null() || values.is_null() {
        return fmi3Status::fmi3Error;
    }
    if n_value_references == 0 {
        return fmi3Status::fmi3Error;
    }

    let placeholder = unsafe { &*(instance as *const Placeholder) };

    let value_refs_slice = unsafe { slice::from_raw_parts(value_references, n_value_references) };
    let input = proto::Fmi3GetClockInputMessage {
        instance_index: placeholder.instance_index,
        value_references: value_refs_slice.iter().map(|&vr| vr as i32).collect(),
        n_value_references: n_value_references as i32,
    };

    let output: proto::Fmi3GetClockOutputMessage = match placeholder.query("fmi3GetClock", &input) {
        Ok(out) => out,
        Err(_) => return fmi3Status::fmi3Fatal,
    };

    let values_slice = unsafe { slice::from_raw_parts_mut(values, n_value_references) };

    let values_count = output.values.len().min(n_value_references);
    for (idx, &val) in output.values.iter().take(values_count).enumerate() {
        values_slice[idx] = if val { 1 } else { 0 };
    }

    proto::Status::try_from(output.status)
        .unwrap_or(proto::Status::Error)
        .into()
}

#[no_mangle]
pub extern "C" fn fmi3SetClock(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *const fmi3Clock,
) -> fmi3Status {
    // Null pointer checks
    if instance.is_null() || value_references.is_null() || values.is_null() {
        return fmi3Status::fmi3Error;
    }
    if n_value_references == 0 {
        return fmi3Status::fmi3Error;
    }

    let placeholder = unsafe { &*(instance as *const Placeholder) };

    let value_refs_slice = unsafe { slice::from_raw_parts(value_references, n_value_references) };
    let values_slice = unsafe { slice::from_raw_parts(values, n_value_references) };

    let input = proto::Fmi3SetClockInputMessage {
        instance_index: placeholder.instance_index,
        value_references: value_refs_slice.iter().map(|&vr| vr as i32).collect(),
        values: values_slice.iter().map(|&val| val != 0).collect(),
        n_value_references: n_value_references as i32,
    };

    let output: proto::Fmi3StatusMessage = match placeholder.query("fmi3SetClock", &input) {
        Ok(out) => out,
        Err(_) => return fmi3Status::fmi3Fatal,
    };

    proto::Status::try_from(output.status)
        .unwrap_or(proto::Status::Error)
        .into()
}
