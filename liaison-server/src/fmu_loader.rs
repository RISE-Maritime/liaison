//! FMU Library Loader
//!
//! This module provides platform-specific dynamic loading of FMU libraries and binding
//! of all FMI 3.0 functions. It uses the `libloading` crate for cross-platform
//! compatibility.
//!
//! The `FmuLibrary` struct encapsulates:
//! - Platform-specific library handle (DLL on Windows, SO on Linux)
//! - Function pointers for all 34 FMI 3.0 functions
//! - Safe Rust wrappers around C function calls
//!
//! # Example
//!
//! ```no_run
//! use liaison_server::fmu_loader::FmuLibrary;
//!
//! let lib_path = "/path/to/fmu/binaries/x86_64-linux/MyModel.so";
//! let fmu = FmuLibrary::new(lib_path)?;
//!
//! // Use the FMI functions

// Allow non-standard naming for FMI compatibility
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
//! let version = (fmu.fmi3_get_version)();
//! ```

use anyhow::{Context, Result};
use libloading::{Library, Symbol};
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::path::Path;

// Import FMI types from liaison-fmi crate
// Note: These types are re-exported from the liaison-fmi crate
type fmi3Instance = *mut c_void;
type fmi3InstanceEnvironment = *mut c_void;
type fmi3ValueReference = u32;
type fmi3Float32 = f32;
type fmi3Float64 = f64;
type fmi3Int8 = i8;
type fmi3UInt8 = u8;
type fmi3Int16 = i16;
type fmi3UInt16 = u16;
type fmi3Int32 = i32;
type fmi3UInt32 = u32;
type fmi3Int64 = i64;
type fmi3UInt64 = u64;
type fmi3Boolean = i32;
type fmi3Char = c_char;
type fmi3String = *const fmi3Char;
type fmi3Byte = u8;
type fmi3Binary = *const fmi3Byte;
type fmi3Clock = i32;
type fmi3FMUState = *mut c_void;

/// FMI 3.0 status codes
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
type fmi3LogMessageCallback = Option<
    unsafe extern "C" fn(
        instanceEnvironment: fmi3InstanceEnvironment,
        status: fmi3Status,
        category: fmi3String,
        message: fmi3String,
    ),
>;

type fmi3IntermediateUpdateCallback = Option<
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

type fmi3ClockUpdateCallback = Option<extern "C" fn(instanceEnvironment: fmi3InstanceEnvironment)>;
type fmi3LockPreemptionCallback = Option<extern "C" fn()>;
type fmi3UnlockPreemptionCallback = Option<extern "C" fn()>;

// FMI 3.0 Function Type Definitions

/// fmi3GetVersion function type
type fmi3GetVersionType = extern "C" fn() -> fmi3String;

/// fmi3SetDebugLogging function type
type fmi3SetDebugLoggingType = extern "C" fn(
    instance: fmi3Instance,
    logging_on: fmi3Boolean,
    n_categories: usize,
    categories: *const fmi3String,
) -> fmi3Status;

/// fmi3InstantiateCoSimulation function type
type fmi3InstantiateCoSimulationType = extern "C" fn(
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
    intermediate_update: fmi3IntermediateUpdateCallback,
) -> fmi3Instance;

/// fmi3InstantiateModelExchange function type
type fmi3InstantiateModelExchangeType = extern "C" fn(
    instance_name: fmi3String,
    instantiation_token: fmi3String,
    resource_path: fmi3String,
    visible: fmi3Boolean,
    logging_on: fmi3Boolean,
    instance_environment: fmi3InstanceEnvironment,
    log_message: fmi3LogMessageCallback,
) -> fmi3Instance;

/// fmi3InstantiateScheduledExecution function type
type fmi3InstantiateScheduledExecutionType = extern "C" fn(
    instance_name: fmi3String,
    instantiation_token: fmi3String,
    resource_path: fmi3String,
    visible: fmi3Boolean,
    logging_on: fmi3Boolean,
    instance_environment: fmi3InstanceEnvironment,
    log_message: fmi3LogMessageCallback,
    clock_update: fmi3ClockUpdateCallback,
    lock_preemption: fmi3LockPreemptionCallback,
    unlock_preemption: fmi3UnlockPreemptionCallback,
) -> fmi3Instance;

/// fmi3FreeInstance function type
type fmi3FreeInstanceType = extern "C" fn(instance: fmi3Instance);

/// fmi3EnterInitializationMode function type
type fmi3EnterInitializationModeType = extern "C" fn(
    instance: fmi3Instance,
    tolerance_defined: fmi3Boolean,
    tolerance: fmi3Float64,
    start_time: fmi3Float64,
    stop_time_defined: fmi3Boolean,
    stop_time: fmi3Float64,
) -> fmi3Status;

/// fmi3ExitInitializationMode function type
type fmi3ExitInitializationModeType = extern "C" fn(instance: fmi3Instance) -> fmi3Status;

/// fmi3EnterEventMode function type
type fmi3EnterEventModeType = extern "C" fn(instance: fmi3Instance) -> fmi3Status;

/// fmi3Terminate function type
type fmi3TerminateType = extern "C" fn(instance: fmi3Instance) -> fmi3Status;

/// fmi3Reset function type
type fmi3ResetType = extern "C" fn(instance: fmi3Instance) -> fmi3Status;

/// fmi3DoStep function type
type fmi3DoStepType = extern "C" fn(
    instance: fmi3Instance,
    current_communication_point: fmi3Float64,
    communication_step_size: fmi3Float64,
    no_set_fmu_state_prior_to_current_point: fmi3Boolean,
    event_handling_needed: *mut fmi3Boolean,
    terminate_simulation: *mut fmi3Boolean,
    early_return: *mut fmi3Boolean,
    last_successful_time: *mut fmi3Float64,
) -> fmi3Status;

// Getter and Setter function types
type fmi3GetFloat32Type = extern "C" fn(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *mut fmi3Float32,
    n_values: usize,
) -> fmi3Status;

type fmi3SetFloat32Type = extern "C" fn(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *const fmi3Float32,
    n_values: usize,
) -> fmi3Status;

type fmi3GetFloat64Type = extern "C" fn(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *mut fmi3Float64,
    n_values: usize,
) -> fmi3Status;

type fmi3SetFloat64Type = extern "C" fn(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *const fmi3Float64,
    n_values: usize,
) -> fmi3Status;

type fmi3GetInt8Type = extern "C" fn(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *mut fmi3Int8,
    n_values: usize,
) -> fmi3Status;

type fmi3SetInt8Type = extern "C" fn(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *const fmi3Int8,
    n_values: usize,
) -> fmi3Status;

type fmi3GetUInt8Type = extern "C" fn(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *mut fmi3UInt8,
    n_values: usize,
) -> fmi3Status;

type fmi3SetUInt8Type = extern "C" fn(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *const fmi3UInt8,
    n_values: usize,
) -> fmi3Status;

type fmi3GetInt16Type = extern "C" fn(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *mut fmi3Int16,
    n_values: usize,
) -> fmi3Status;

type fmi3SetInt16Type = extern "C" fn(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *const fmi3Int16,
    n_values: usize,
) -> fmi3Status;

type fmi3GetUInt16Type = extern "C" fn(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *mut fmi3UInt16,
    n_values: usize,
) -> fmi3Status;

type fmi3SetUInt16Type = extern "C" fn(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *const fmi3UInt16,
    n_values: usize,
) -> fmi3Status;

type fmi3GetInt32Type = extern "C" fn(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *mut fmi3Int32,
    n_values: usize,
) -> fmi3Status;

type fmi3SetInt32Type = extern "C" fn(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *const fmi3Int32,
    n_values: usize,
) -> fmi3Status;

type fmi3GetUInt32Type = extern "C" fn(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *mut fmi3UInt32,
    n_values: usize,
) -> fmi3Status;

type fmi3SetUInt32Type = extern "C" fn(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *const fmi3UInt32,
    n_values: usize,
) -> fmi3Status;

type fmi3GetInt64Type = extern "C" fn(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *mut fmi3Int64,
    n_values: usize,
) -> fmi3Status;

type fmi3SetInt64Type = extern "C" fn(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *const fmi3Int64,
    n_values: usize,
) -> fmi3Status;

type fmi3GetUInt64Type = extern "C" fn(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *mut fmi3UInt64,
    n_values: usize,
) -> fmi3Status;

type fmi3SetUInt64Type = extern "C" fn(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *const fmi3UInt64,
    n_values: usize,
) -> fmi3Status;

type fmi3GetBooleanType = extern "C" fn(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *mut fmi3Boolean,
    n_values: usize,
) -> fmi3Status;

type fmi3SetBooleanType = extern "C" fn(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *const fmi3Boolean,
    n_values: usize,
) -> fmi3Status;

type fmi3GetStringType = extern "C" fn(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *mut fmi3String,
    n_values: usize,
) -> fmi3Status;

type fmi3SetStringType = extern "C" fn(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *const fmi3String,
    n_values: usize,
) -> fmi3Status;

type fmi3GetBinaryType = extern "C" fn(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    value_sizes: *mut usize,
    values: *mut fmi3Binary,
    n_values: usize,
) -> fmi3Status;

type fmi3SetBinaryType = extern "C" fn(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    value_sizes: *const usize,
    values: *const fmi3Binary,
    n_values: usize,
) -> fmi3Status;

type fmi3GetClockType = extern "C" fn(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *mut fmi3Clock,
) -> fmi3Status;

type fmi3SetClockType = extern "C" fn(
    instance: fmi3Instance,
    value_references: *const fmi3ValueReference,
    n_value_references: usize,
    values: *const fmi3Clock,
) -> fmi3Status;

/// FMU Library wrapper that holds the dynamic library handle and all FMI 3.0 function pointers.
///
/// This struct provides safe access to the FMI functions loaded from a platform-specific
/// shared library (.dll on Windows, .so on Linux).
pub struct FmuLibrary {
    /// The underlying library handle (kept alive for the duration of the struct)
    #[allow(dead_code)]
    pub(crate) library: Library,

    // Common Functions
    pub fmi3_get_version: fmi3GetVersionType,
    pub fmi3_set_debug_logging: fmi3SetDebugLoggingType,

    // Instantiation Functions
    pub fmi3_instantiate_co_simulation: fmi3InstantiateCoSimulationType,
    pub fmi3_instantiate_model_exchange: fmi3InstantiateModelExchangeType,
    pub fmi3_instantiate_scheduled_execution: fmi3InstantiateScheduledExecutionType,
    pub fmi3_free_instance: fmi3FreeInstanceType,

    // Lifecycle Functions
    pub fmi3_enter_initialization_mode: fmi3EnterInitializationModeType,
    pub fmi3_exit_initialization_mode: fmi3ExitInitializationModeType,
    pub fmi3_enter_event_mode: fmi3EnterEventModeType,
    pub fmi3_terminate: fmi3TerminateType,
    pub fmi3_reset: fmi3ResetType,

    // Co-Simulation Function
    pub fmi3_do_step: fmi3DoStepType,

    // Float32 Functions
    pub fmi3_get_float32: fmi3GetFloat32Type,
    pub fmi3_set_float32: fmi3SetFloat32Type,

    // Float64 Functions
    pub fmi3_get_float64: fmi3GetFloat64Type,
    pub fmi3_set_float64: fmi3SetFloat64Type,

    // Int8 Functions
    pub fmi3_get_int8: fmi3GetInt8Type,
    pub fmi3_set_int8: fmi3SetInt8Type,

    // UInt8 Functions
    pub fmi3_get_uint8: fmi3GetUInt8Type,
    pub fmi3_set_uint8: fmi3SetUInt8Type,

    // Int16 Functions
    pub fmi3_get_int16: fmi3GetInt16Type,
    pub fmi3_set_int16: fmi3SetInt16Type,

    // UInt16 Functions
    pub fmi3_get_uint16: fmi3GetUInt16Type,
    pub fmi3_set_uint16: fmi3SetUInt16Type,

    // Int32 Functions
    pub fmi3_get_int32: fmi3GetInt32Type,
    pub fmi3_set_int32: fmi3SetInt32Type,

    // UInt32 Functions
    pub fmi3_get_uint32: fmi3GetUInt32Type,
    pub fmi3_set_uint32: fmi3SetUInt32Type,

    // Int64 Functions
    pub fmi3_get_int64: fmi3GetInt64Type,
    pub fmi3_set_int64: fmi3SetInt64Type,

    // UInt64 Functions
    pub fmi3_get_uint64: fmi3GetUInt64Type,
    pub fmi3_set_uint64: fmi3SetUInt64Type,

    // Boolean Functions
    pub fmi3_get_boolean: fmi3GetBooleanType,
    pub fmi3_set_boolean: fmi3SetBooleanType,

    // String Functions
    pub fmi3_get_string: fmi3GetStringType,
    pub fmi3_set_string: fmi3SetStringType,

    // Binary Functions
    pub fmi3_get_binary: fmi3GetBinaryType,
    pub fmi3_set_binary: fmi3SetBinaryType,

    // Clock Functions
    pub fmi3_get_clock: fmi3GetClockType,
    pub fmi3_set_clock: fmi3SetClockType,
}

impl FmuLibrary {
    /// Creates a new FmuLibrary by loading the library at the given path.
    ///
    /// # Arguments
    ///
    /// * `lib_path` - Path to the FMU shared library file (.dll on Windows, .so on Linux)
    ///
    /// # Returns
    ///
    /// A Result containing the FmuLibrary on success, or an error if:
    /// - The library file cannot be found
    /// - The library cannot be loaded
    /// - Any required FMI function cannot be found in the library
    ///
    /// # Example
    ///
    /// ```no_run
    /// use liaison_server::fmu_loader::FmuLibrary;
    ///
    /// let fmu = FmuLibrary::new("/path/to/model.so")?;
    /// ```
    pub fn new<P: AsRef<Path>>(lib_path: P) -> Result<Self> {
        let lib_path = lib_path.as_ref();

        // Load the library with detailed error reporting
        let library = unsafe {
            Library::new(lib_path).with_context(|| {
                format!(
                    "Failed to load FMU library from path: {}",
                    lib_path.display()
                )
            })?
        };

        // Helper macro to load a function symbol with error handling
        macro_rules! load_symbol {
            ($name:expr, $type:ty) => {{
                unsafe {
                    let symbol: Symbol<$type> = library
                        .get($name)
                        .with_context(|| format!("Failed to load function {:?} from FMU library", $name))?;
                    *symbol
                }
            }};
        }

        // Load all FMI 3.0 functions BEFORE constructing the struct
        // This is necessary because we need to borrow `library` to load symbols,
        // but then move `library` into the struct to keep it alive.

        // Common Functions
        let fmi3_get_version = load_symbol!(b"fmi3GetVersion\0", fmi3GetVersionType);
        let fmi3_set_debug_logging = load_symbol!(b"fmi3SetDebugLogging\0", fmi3SetDebugLoggingType);

        // Instantiation Functions
        let fmi3_instantiate_co_simulation = load_symbol!(
            b"fmi3InstantiateCoSimulation\0",
            fmi3InstantiateCoSimulationType
        );
        let fmi3_instantiate_model_exchange = load_symbol!(
            b"fmi3InstantiateModelExchange\0",
            fmi3InstantiateModelExchangeType
        );
        let fmi3_instantiate_scheduled_execution = load_symbol!(
            b"fmi3InstantiateScheduledExecution\0",
            fmi3InstantiateScheduledExecutionType
        );
        let fmi3_free_instance = load_symbol!(b"fmi3FreeInstance\0", fmi3FreeInstanceType);

        // Lifecycle Functions
        let fmi3_enter_initialization_mode = load_symbol!(
            b"fmi3EnterInitializationMode\0",
            fmi3EnterInitializationModeType
        );
        let fmi3_exit_initialization_mode = load_symbol!(
            b"fmi3ExitInitializationMode\0",
            fmi3ExitInitializationModeType
        );
        let fmi3_enter_event_mode = load_symbol!(b"fmi3EnterEventMode\0", fmi3EnterEventModeType);
        let fmi3_terminate = load_symbol!(b"fmi3Terminate\0", fmi3TerminateType);
        let fmi3_reset = load_symbol!(b"fmi3Reset\0", fmi3ResetType);

        // Co-Simulation Function
        let fmi3_do_step = load_symbol!(b"fmi3DoStep\0", fmi3DoStepType);

        // Float32 Functions
        let fmi3_get_float32 = load_symbol!(b"fmi3GetFloat32\0", fmi3GetFloat32Type);
        let fmi3_set_float32 = load_symbol!(b"fmi3SetFloat32\0", fmi3SetFloat32Type);

        // Float64 Functions
        let fmi3_get_float64 = load_symbol!(b"fmi3GetFloat64\0", fmi3GetFloat64Type);
        let fmi3_set_float64 = load_symbol!(b"fmi3SetFloat64\0", fmi3SetFloat64Type);

        // Int8 Functions
        let fmi3_get_int8 = load_symbol!(b"fmi3GetInt8\0", fmi3GetInt8Type);
        let fmi3_set_int8 = load_symbol!(b"fmi3SetInt8\0", fmi3SetInt8Type);

        // UInt8 Functions
        let fmi3_get_uint8 = load_symbol!(b"fmi3GetUInt8\0", fmi3GetUInt8Type);
        let fmi3_set_uint8 = load_symbol!(b"fmi3SetUInt8\0", fmi3SetUInt8Type);

        // Int16 Functions
        let fmi3_get_int16 = load_symbol!(b"fmi3GetInt16\0", fmi3GetInt16Type);
        let fmi3_set_int16 = load_symbol!(b"fmi3SetInt16\0", fmi3SetInt16Type);

        // UInt16 Functions
        let fmi3_get_uint16 = load_symbol!(b"fmi3GetUInt16\0", fmi3GetUInt16Type);
        let fmi3_set_uint16 = load_symbol!(b"fmi3SetUInt16\0", fmi3SetUInt16Type);

        // Int32 Functions
        let fmi3_get_int32 = load_symbol!(b"fmi3GetInt32\0", fmi3GetInt32Type);
        let fmi3_set_int32 = load_symbol!(b"fmi3SetInt32\0", fmi3SetInt32Type);

        // UInt32 Functions
        let fmi3_get_uint32 = load_symbol!(b"fmi3GetUInt32\0", fmi3GetUInt32Type);
        let fmi3_set_uint32 = load_symbol!(b"fmi3SetUInt32\0", fmi3SetUInt32Type);

        // Int64 Functions
        let fmi3_get_int64 = load_symbol!(b"fmi3GetInt64\0", fmi3GetInt64Type);
        let fmi3_set_int64 = load_symbol!(b"fmi3SetInt64\0", fmi3SetInt64Type);

        // UInt64 Functions
        let fmi3_get_uint64 = load_symbol!(b"fmi3GetUInt64\0", fmi3GetUInt64Type);
        let fmi3_set_uint64 = load_symbol!(b"fmi3SetUInt64\0", fmi3SetUInt64Type);

        // Boolean Functions
        let fmi3_get_boolean = load_symbol!(b"fmi3GetBoolean\0", fmi3GetBooleanType);
        let fmi3_set_boolean = load_symbol!(b"fmi3SetBoolean\0", fmi3SetBooleanType);

        // String Functions
        let fmi3_get_string = load_symbol!(b"fmi3GetString\0", fmi3GetStringType);
        let fmi3_set_string = load_symbol!(b"fmi3SetString\0", fmi3SetStringType);

        // Binary Functions
        let fmi3_get_binary = load_symbol!(b"fmi3GetBinary\0", fmi3GetBinaryType);
        let fmi3_set_binary = load_symbol!(b"fmi3SetBinary\0", fmi3SetBinaryType);

        // Clock Functions
        let fmi3_get_clock = load_symbol!(b"fmi3GetClock\0", fmi3GetClockType);
        let fmi3_set_clock = load_symbol!(b"fmi3SetClock\0", fmi3SetClockType);

        // Now construct the struct with all loaded symbols
        Ok(Self {
            // Store the library handle to keep it alive
            library,

            // Common Functions
            fmi3_get_version,
            fmi3_set_debug_logging,

            // Instantiation Functions
            fmi3_instantiate_co_simulation,
            fmi3_instantiate_model_exchange,
            fmi3_instantiate_scheduled_execution,
            fmi3_free_instance,

            // Lifecycle Functions
            fmi3_enter_initialization_mode,
            fmi3_exit_initialization_mode,
            fmi3_enter_event_mode,
            fmi3_terminate,
            fmi3_reset,

            // Co-Simulation Function
            fmi3_do_step,

            // Float32 Functions
            fmi3_get_float32,
            fmi3_set_float32,

            // Float64 Functions
            fmi3_get_float64,
            fmi3_set_float64,

            // Int8 Functions
            fmi3_get_int8,
            fmi3_set_int8,

            // UInt8 Functions
            fmi3_get_uint8,
            fmi3_set_uint8,

            // Int16 Functions
            fmi3_get_int16,
            fmi3_set_int16,

            // UInt16 Functions
            fmi3_get_uint16,
            fmi3_set_uint16,

            // Int32 Functions
            fmi3_get_int32,
            fmi3_set_int32,

            // UInt32 Functions
            fmi3_get_uint32,
            fmi3_set_uint32,

            // Int64 Functions
            fmi3_get_int64,
            fmi3_set_int64,

            // UInt64 Functions
            fmi3_get_uint64,
            fmi3_set_uint64,

            // Boolean Functions
            fmi3_get_boolean,
            fmi3_set_boolean,

            // String Functions
            fmi3_get_string,
            fmi3_set_string,

            // Binary Functions
            fmi3_get_binary,
            fmi3_set_binary,

            // Clock Functions
            fmi3_get_clock,
            fmi3_set_clock,
        })
    }
}

// Implement Debug for FmuLibrary
impl std::fmt::Debug for FmuLibrary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Get FMI version string to include in debug output
        let version_ptr = (self.fmi3_get_version)();
        let version_str = if !version_ptr.is_null() {
            unsafe {
                CStr::from_ptr(version_ptr)
                    .to_str()
                    .unwrap_or("<invalid UTF-8>")
            }
        } else {
            "<null>"
        };

        f.debug_struct("FmuLibrary")
            .field("fmi_version", &version_str)
            .field("has_fmi3_get_version", &true)
            .field("has_fmi3_set_debug_logging", &true)
            .field("has_fmi3_instantiate_co_simulation", &true)
            .field("has_fmi3_instantiate_model_exchange", &true)
            .field("has_fmi3_instantiate_scheduled_execution", &true)
            .field("has_fmi3_free_instance", &true)
            .field("has_fmi3_enter_initialization_mode", &true)
            .field("has_fmi3_exit_initialization_mode", &true)
            .field("has_fmi3_enter_event_mode", &true)
            .field("has_fmi3_terminate", &true)
            .field("has_fmi3_reset", &true)
            .field("has_fmi3_do_step", &true)
            .finish_non_exhaustive()
    }
}

// The Drop implementation is automatic - when FmuLibrary goes out of scope,
// the Library will be dropped and unloaded automatically.

/// Constructs the platform-specific library path for an FMU.
///
/// Based on the C++ implementation in liaison.cpp lines 743-753, this function
/// constructs the path to the binary within an extracted FMU archive.
///
/// # Arguments
///
/// * `temp_path` - The path to the extracted FMU directory
/// * `model_name` - The name of the model (without extension)
///
/// # Returns
///
/// A String containing the full path to the platform-specific shared library.
///
/// # Platform-specific paths
///
/// - Windows (64-bit): `{temp_path}/binaries/x86_64-windows/{model_name}.dll`
/// - Windows (32-bit): `{temp_path}/binaries/x86-windows/{model_name}.dll`
/// - Linux (64-bit): `{temp_path}/binaries/x86_64-linux/{model_name}.so`
///
/// # Example
///
/// ```
/// use liaison_server::fmu_loader::construct_library_path;
///
/// let path = construct_library_path("/tmp/fmu_extract", "BouncingBall");
/// // On Linux: "/tmp/fmu_extract/binaries/x86_64-linux/BouncingBall.so"
/// // On Windows 64-bit: "/tmp/fmu_extract/binaries/x86_64-windows/BouncingBall.dll"
/// ```
pub fn construct_library_path(temp_path: &str, model_name: &str) -> String {
    #[cfg(all(target_os = "windows", target_pointer_width = "64"))]
    {
        format!("{}/binaries/x86_64-windows/{}.dll", temp_path, model_name)
    }

    #[cfg(all(target_os = "windows", target_pointer_width = "32"))]
    {
        format!("{}/binaries/x86-windows/{}.dll", temp_path, model_name)
    }

    #[cfg(target_os = "linux")]
    {
        format!("{}/binaries/x86_64-linux/{}.so", temp_path, model_name)
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        compile_error!("Unsupported platform - only Windows and Linux are supported")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    // ============================================================================
    // Platform-Specific Library Path Construction Tests
    // ============================================================================

    /// Test that library path construction works correctly for the current platform.
    #[test]
    fn test_construct_library_path() {
        let path = construct_library_path("/tmp/fmu", "MyModel");

        #[cfg(all(target_os = "windows", target_pointer_width = "64"))]
        assert_eq!(path, "/tmp/fmu/binaries/x86_64-windows/MyModel.dll");

        #[cfg(all(target_os = "windows", target_pointer_width = "32"))]
        assert_eq!(path, "/tmp/fmu/binaries/x86-windows/MyModel.dll");

        #[cfg(target_os = "linux")]
        assert_eq!(path, "/tmp/fmu/binaries/x86_64-linux/MyModel.so");
    }

    /// Test library path construction with special characters in paths and names.
    #[test]
    fn test_construct_library_path_with_special_chars() {
        let path = construct_library_path("/tmp/my_fmu_123", "Model-v2.0");

        #[cfg(target_os = "linux")]
        assert_eq!(
            path,
            "/tmp/my_fmu_123/binaries/x86_64-linux/Model-v2.0.so"
        );

        #[cfg(all(target_os = "windows", target_pointer_width = "64"))]
        assert_eq!(
            path,
            "/tmp/my_fmu_123/binaries/x86_64-windows/Model-v2.0.dll"
        );
    }

    /// Test library path construction with empty strings (edge case).
    #[test]
    fn test_construct_library_path_empty() {
        let path = construct_library_path("", "Model");

        #[cfg(target_os = "linux")]
        assert_eq!(path, "/binaries/x86_64-linux/Model.so");

        #[cfg(all(target_os = "windows", target_pointer_width = "64"))]
        assert_eq!(path, "/binaries/x86_64-windows/Model.dll");
    }

    /// Test library path construction with paths containing spaces.
    #[test]
    fn test_construct_library_path_with_spaces() {
        let path = construct_library_path("/tmp/my fmu dir", "My Model");

        #[cfg(target_os = "linux")]
        assert_eq!(path, "/tmp/my fmu dir/binaries/x86_64-linux/My Model.so");

        #[cfg(all(target_os = "windows", target_pointer_width = "64"))]
        assert_eq!(
            path,
            "/tmp/my fmu dir/binaries/x86_64-windows/My Model.dll"
        );
    }

    /// Test library path construction with absolute paths.
    #[test]
    fn test_construct_library_path_absolute() {
        #[cfg(target_os = "linux")]
        {
            let path = construct_library_path("/home/user/fmus/extract", "BouncingBall");
            assert_eq!(
                path,
                "/home/user/fmus/extract/binaries/x86_64-linux/BouncingBall.so"
            );
        }

        #[cfg(all(target_os = "windows", target_pointer_width = "64"))]
        {
            let path = construct_library_path("C:/Users/test/fmus", "BouncingBall");
            assert_eq!(
                path,
                "C:/Users/test/fmus/binaries/x86_64-windows/BouncingBall.dll"
            );
        }
    }

    // ============================================================================
    // Error Handling Tests
    // ============================================================================

    /// Test that loading a non-existent library file returns an appropriate error.
    #[test]
    fn test_load_nonexistent_library() {
        let nonexistent_path = "/tmp/nonexistent_library_12345.so";
        let result = FmuLibrary::new(nonexistent_path);

        assert!(result.is_err(), "Expected error when loading non-existent library");

        let error = result.unwrap_err();
        let error_msg = format!("{:#}", error);

        // Verify that the error message contains useful information
        assert!(
            error_msg.contains("Failed to load FMU library") ||
            error_msg.contains("nonexistent_library"),
            "Error message should mention the failed library load: {}",
            error_msg
        );
    }

    /// Test that loading a directory instead of a file returns an error.
    #[test]
    fn test_load_directory_instead_of_file() {
        let temp_dir = TempDir::new().unwrap();
        let result = FmuLibrary::new(temp_dir.path());

        assert!(result.is_err(), "Expected error when loading a directory");

        let error = result.unwrap_err();
        let error_msg = format!("{:#}", error);

        // The error should indicate a problem loading the library
        assert!(
            error_msg.contains("Failed to load FMU library"),
            "Error message should indicate library loading failure: {}",
            error_msg
        );
    }

    /// Test that loading a file with incorrect format returns an error.
    #[test]
    fn test_load_invalid_library_format() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_lib_path = temp_dir.path().join("invalid.so");

        // Create a text file instead of a valid shared library
        let mut file = fs::File::create(&invalid_lib_path).unwrap();
        writeln!(file, "This is not a valid shared library").unwrap();

        let result = FmuLibrary::new(&invalid_lib_path);

        assert!(result.is_err(), "Expected error when loading invalid library format");

        let error = result.unwrap_err();
        let error_msg = format!("{:#}", error);

        // Error should indicate the library couldn't be loaded
        assert!(
            error_msg.contains("Failed to load FMU library"),
            "Error message should indicate library loading failure: {}",
            error_msg
        );
    }

    /// Test that loading an empty file returns an error.
    #[test]
    fn test_load_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let empty_file_path = temp_dir.path().join("empty.so");

        // Create an empty file
        fs::File::create(&empty_file_path).unwrap();

        let result = FmuLibrary::new(&empty_file_path);

        assert!(result.is_err(), "Expected error when loading empty file");
    }

    // ============================================================================
    // Mock Library Tests
    // ============================================================================

    /// Creates a mock C source file that implements minimal FMI 3.0 functions.
    /// This is used to test the loader without requiring a real FMU.
    #[cfg(target_os = "linux")]
    fn create_mock_fmu_source() -> &'static str {
        r#"
#include <stddef.h>
#include <stdint.h>

// FMI 3.0 type definitions
typedef void* fmi3Instance;
typedef void* fmi3InstanceEnvironment;
typedef uint32_t fmi3ValueReference;
typedef float fmi3Float32;
typedef double fmi3Float64;
typedef int8_t fmi3Int8;
typedef uint8_t fmi3UInt8;
typedef int16_t fmi3Int16;
typedef uint16_t fmi3UInt16;
typedef int32_t fmi3Int32;
typedef uint32_t fmi3UInt32;
typedef int64_t fmi3Int64;
typedef uint64_t fmi3UInt64;
typedef int32_t fmi3Boolean;
typedef char fmi3Char;
typedef const fmi3Char* fmi3String;
typedef uint8_t fmi3Byte;
typedef const fmi3Byte* fmi3Binary;
typedef int32_t fmi3Clock;
typedef void* fmi3FMUState;

typedef enum {
    fmi3OK = 0,
    fmi3Warning = 1,
    fmi3Discard = 2,
    fmi3Error = 3,
    fmi3Fatal = 4
} fmi3Status;

// Mock FMI 3.0 functions
const char* fmi3GetVersion(void) {
    return "3.0";
}

fmi3Status fmi3SetDebugLogging(fmi3Instance instance, fmi3Boolean loggingOn,
                                size_t nCategories, const fmi3String categories[]) {
    return fmi3OK;
}

fmi3Instance fmi3InstantiateCoSimulation(
    fmi3String instanceName, fmi3String instantiationToken,
    fmi3String resourcePath, fmi3Boolean visible, fmi3Boolean loggingOn,
    fmi3Boolean eventModeUsed, fmi3Boolean earlyReturnAllowed,
    const fmi3ValueReference requiredIntermediateVariables[],
    size_t nRequiredIntermediateVariables, fmi3InstanceEnvironment instanceEnvironment,
    void* logMessage, void* intermediateUpdate) {
    return (fmi3Instance)0x1234;
}

fmi3Instance fmi3InstantiateModelExchange(
    fmi3String instanceName, fmi3String instantiationToken,
    fmi3String resourcePath, fmi3Boolean visible, fmi3Boolean loggingOn,
    fmi3InstanceEnvironment instanceEnvironment, void* logMessage) {
    return (fmi3Instance)0x1234;
}

fmi3Instance fmi3InstantiateScheduledExecution(
    fmi3String instanceName, fmi3String instantiationToken,
    fmi3String resourcePath, fmi3Boolean visible, fmi3Boolean loggingOn,
    fmi3InstanceEnvironment instanceEnvironment, void* logMessage,
    void* clockUpdate, void* lockPreemption, void* unlockPreemption) {
    return (fmi3Instance)0x1234;
}

void fmi3FreeInstance(fmi3Instance instance) {}

fmi3Status fmi3EnterInitializationMode(fmi3Instance instance, fmi3Boolean toleranceDefined,
                                       fmi3Float64 tolerance, fmi3Float64 startTime,
                                       fmi3Boolean stopTimeDefined, fmi3Float64 stopTime) {
    return fmi3OK;
}

fmi3Status fmi3ExitInitializationMode(fmi3Instance instance) {
    return fmi3OK;
}

fmi3Status fmi3EnterEventMode(fmi3Instance instance) {
    return fmi3OK;
}

fmi3Status fmi3Terminate(fmi3Instance instance) {
    return fmi3OK;
}

fmi3Status fmi3Reset(fmi3Instance instance) {
    return fmi3OK;
}

fmi3Status fmi3DoStep(fmi3Instance instance, fmi3Float64 currentCommunicationPoint,
                      fmi3Float64 communicationStepSize, fmi3Boolean noSetFMUStatePriorToCurrentPoint,
                      fmi3Boolean* eventHandlingNeeded, fmi3Boolean* terminateSimulation,
                      fmi3Boolean* earlyReturn, fmi3Float64* lastSuccessfulTime) {
    return fmi3OK;
}

// Getter/Setter functions
fmi3Status fmi3GetFloat32(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                          fmi3Float32 value[], size_t nValues) {
    return fmi3OK;
}

fmi3Status fmi3SetFloat32(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                          const fmi3Float32 value[], size_t nValues) {
    return fmi3OK;
}

fmi3Status fmi3GetFloat64(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                          fmi3Float64 value[], size_t nValues) {
    return fmi3OK;
}

fmi3Status fmi3SetFloat64(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                          const fmi3Float64 value[], size_t nValues) {
    return fmi3OK;
}

fmi3Status fmi3GetInt8(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                       fmi3Int8 value[], size_t nValues) {
    return fmi3OK;
}

fmi3Status fmi3SetInt8(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                       const fmi3Int8 value[], size_t nValues) {
    return fmi3OK;
}

fmi3Status fmi3GetUInt8(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                        fmi3UInt8 value[], size_t nValues) {
    return fmi3OK;
}

fmi3Status fmi3SetUInt8(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                        const fmi3UInt8 value[], size_t nValues) {
    return fmi3OK;
}

fmi3Status fmi3GetInt16(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                        fmi3Int16 value[], size_t nValues) {
    return fmi3OK;
}

fmi3Status fmi3SetInt16(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                        const fmi3Int16 value[], size_t nValues) {
    return fmi3OK;
}

fmi3Status fmi3GetUInt16(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                         fmi3UInt16 value[], size_t nValues) {
    return fmi3OK;
}

fmi3Status fmi3SetUInt16(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                         const fmi3UInt16 value[], size_t nValues) {
    return fmi3OK;
}

fmi3Status fmi3GetInt32(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                        fmi3Int32 value[], size_t nValues) {
    return fmi3OK;
}

fmi3Status fmi3SetInt32(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                        const fmi3Int32 value[], size_t nValues) {
    return fmi3OK;
}

fmi3Status fmi3GetUInt32(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                         fmi3UInt32 value[], size_t nValues) {
    return fmi3OK;
}

fmi3Status fmi3SetUInt32(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                         const fmi3UInt32 value[], size_t nValues) {
    return fmi3OK;
}

fmi3Status fmi3GetInt64(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                        fmi3Int64 value[], size_t nValues) {
    return fmi3OK;
}

fmi3Status fmi3SetInt64(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                        const fmi3Int64 value[], size_t nValues) {
    return fmi3OK;
}

fmi3Status fmi3GetUInt64(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                         fmi3UInt64 value[], size_t nValues) {
    return fmi3OK;
}

fmi3Status fmi3SetUInt64(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                         const fmi3UInt64 value[], size_t nValues) {
    return fmi3OK;
}

fmi3Status fmi3GetBoolean(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                          fmi3Boolean value[], size_t nValues) {
    return fmi3OK;
}

fmi3Status fmi3SetBoolean(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                          const fmi3Boolean value[], size_t nValues) {
    return fmi3OK;
}

fmi3Status fmi3GetString(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                         fmi3String value[], size_t nValues) {
    return fmi3OK;
}

fmi3Status fmi3SetString(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                         const fmi3String value[], size_t nValues) {
    return fmi3OK;
}

fmi3Status fmi3GetBinary(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                         size_t valueSizes[], fmi3Binary value[], size_t nValues) {
    return fmi3OK;
}

fmi3Status fmi3SetBinary(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                         const size_t valueSizes[], const fmi3Binary value[], size_t nValues) {
    return fmi3OK;
}

fmi3Status fmi3GetClock(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                        fmi3Clock value[]) {
    return fmi3OK;
}

fmi3Status fmi3SetClock(fmi3Instance instance, const fmi3ValueReference vr[], size_t nvr,
                        const fmi3Clock value[]) {
    return fmi3OK;
}
"#
    }

    /// Test loading a mock FMU library that implements all required functions.
    /// This test compiles a minimal FMI 3.0 library and verifies all function pointers load correctly.
    #[test]
    #[cfg(target_os = "linux")]
    fn test_load_mock_fmu_library() {
        use std::process::Command;

        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("mock_fmu.c");
        let lib_path = temp_dir.path().join("libmock_fmu.so");

        // Write the mock FMU source
        let mut source_file = fs::File::create(&source_path).unwrap();
        write!(source_file, "{}", create_mock_fmu_source()).unwrap();

        // Compile the mock FMU library using gcc
        let compile_output = Command::new("gcc")
            .args(&[
                "-shared",
                "-fPIC",
                "-o",
                lib_path.to_str().unwrap(),
                source_path.to_str().unwrap(),
            ])
            .output();

        // Skip test if gcc is not available
        if compile_output.is_err() {
            eprintln!("Skipping test: gcc not available");
            return;
        }

        let output = compile_output.unwrap();
        if !output.status.success() {
            panic!(
                "Failed to compile mock FMU: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        // Now test loading the mock library
        let fmu_lib = FmuLibrary::new(&lib_path).expect("Failed to load mock FMU library");

        // Verify the version function works
        let version_ptr = (fmu_lib.fmi3_get_version)();
        assert!(!version_ptr.is_null(), "fmi3GetVersion returned null");

        let version_str = unsafe { CStr::from_ptr(version_ptr).to_str().unwrap() };
        assert_eq!(version_str, "3.0", "Expected FMI version 3.0");
    }

    /// Test that Debug implementation works correctly for FmuLibrary.
    #[test]
    #[cfg(target_os = "linux")]
    fn test_fmu_library_debug_output() {
        use std::process::Command;

        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("mock_fmu.c");
        let lib_path = temp_dir.path().join("libmock_fmu.so");

        // Write and compile mock FMU
        let mut source_file = fs::File::create(&source_path).unwrap();
        write!(source_file, "{}", create_mock_fmu_source()).unwrap();

        let compile_output = Command::new("gcc")
            .args(&[
                "-shared",
                "-fPIC",
                "-o",
                lib_path.to_str().unwrap(),
                source_path.to_str().unwrap(),
            ])
            .output();

        if compile_output.is_err() {
            eprintln!("Skipping test: gcc not available");
            return;
        }

        let output = compile_output.unwrap();
        if !output.status.success() {
            return;
        }

        let fmu_lib = FmuLibrary::new(&lib_path).expect("Failed to load mock FMU library");

        // Test Debug output
        let debug_output = format!("{:?}", fmu_lib);

        assert!(debug_output.contains("FmuLibrary"), "Debug output should contain struct name");
        assert!(debug_output.contains("fmi_version"), "Debug output should contain version");
        assert!(debug_output.contains("3.0"), "Debug output should show FMI version 3.0");
    }

    // ============================================================================
    // Function Pointer Validation Tests
    // ============================================================================

    /// Test that a library missing required functions fails to load with appropriate error.
    #[test]
    #[cfg(target_os = "linux")]
    fn test_load_incomplete_fmu_library() {
        use std::process::Command;

        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("incomplete_fmu.c");
        let lib_path = temp_dir.path().join("libincomplete_fmu.so");

        // Create a minimal library with only a subset of required functions
        let incomplete_source = r#"
const char* fmi3GetVersion(void) {
    return "3.0";
}
// Missing all other required FMI functions
"#;

        let mut source_file = fs::File::create(&source_path).unwrap();
        write!(source_file, "{}", incomplete_source).unwrap();

        let compile_output = Command::new("gcc")
            .args(&[
                "-shared",
                "-fPIC",
                "-o",
                lib_path.to_str().unwrap(),
                source_path.to_str().unwrap(),
            ])
            .output();

        if compile_output.is_err() {
            eprintln!("Skipping test: gcc not available");
            return;
        }

        let output = compile_output.unwrap();
        if !output.status.success() {
            return;
        }

        // Attempt to load the incomplete library - should fail
        let result = FmuLibrary::new(&lib_path);

        assert!(
            result.is_err(),
            "Expected error when loading incomplete FMU library"
        );

        let error = result.unwrap_err();
        let error_msg = format!("{:#}", error);

        // Error should mention which function failed to load
        assert!(
            error_msg.contains("Failed to load function") || error_msg.contains("fmi3"),
            "Error should mention failed function: {}",
            error_msg
        );
    }

    // ============================================================================
    // Status Code Tests
    // ============================================================================

    /// Test that fmi3Status enum values match FMI 3.0 specification.
    #[test]
    fn test_fmi3_status_values() {
        assert_eq!(fmi3Status::fmi3OK as i32, 0);
        assert_eq!(fmi3Status::fmi3Warning as i32, 1);
        assert_eq!(fmi3Status::fmi3Discard as i32, 2);
        assert_eq!(fmi3Status::fmi3Error as i32, 3);
        assert_eq!(fmi3Status::fmi3Fatal as i32, 4);
    }

    /// Test that fmi3Status is Copy and Clone.
    #[test]
    fn test_fmi3_status_copy_clone() {
        let status1 = fmi3Status::fmi3OK;
        let status2 = status1; // Copy
        let status3 = status1.clone(); // Clone

        assert_eq!(status1, status2);
        assert_eq!(status1, status3);
    }

    /// Test that fmi3Status supports Debug formatting.
    #[test]
    fn test_fmi3_status_debug() {
        let status = fmi3Status::fmi3OK;
        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("fmi3OK"));
    }

    // ============================================================================
    // Integration Tests
    // ============================================================================

    /// Test the complete workflow: construct path and attempt to load.
    #[test]
    fn test_workflow_construct_and_load() {
        let temp_path = "/tmp/test_fmu";
        let model_name = "TestModel";

        // Construct the library path
        let lib_path = construct_library_path(temp_path, model_name);

        // Verify path is constructed correctly
        #[cfg(target_os = "linux")]
        assert!(lib_path.contains("x86_64-linux"));

        // Attempt to load (will fail since file doesn't exist, but tests the workflow)
        let result = FmuLibrary::new(&lib_path);
        assert!(result.is_err(), "Expected error for non-existent library");
    }

    /// Test that library path uses forward slashes consistently.
    #[test]
    fn test_library_path_uses_forward_slashes() {
        let path = construct_library_path("/tmp/test", "Model");

        // Count forward slashes
        let forward_slashes = path.matches('/').count();
        assert!(forward_slashes >= 3, "Path should contain multiple forward slashes");

        // Ensure no backslashes (even on Windows, we use forward slashes)
        assert!(!path.contains('\\'), "Path should not contain backslashes");
    }
}
