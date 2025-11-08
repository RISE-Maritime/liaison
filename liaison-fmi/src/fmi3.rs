// FMI 3.0 function definitions and exports
// This module provides the C ABI exports for the FMI functions

use std::os::raw::{c_char, c_void};

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

pub type fmi3ClockUpdateCallback = Option<extern "C" fn(instanceEnvironment: fmi3InstanceEnvironment)>;
pub type fmi3LockPreemptionCallback = Option<extern "C" fn()>;
pub type fmi3UnlockPreemptionCallback = Option<extern "C" fn()>;

// Stub implementations - these will be implemented in subsequent steps
#[no_mangle]
pub extern "C" fn fmi3GetVersion() -> fmi3String {
    c"3.0".as_ptr() as fmi3String
}

#[no_mangle]
pub extern "C" fn fmi3SetDebugLogging(
    _instance: fmi3Instance,
    _logging_on: fmi3Boolean,
    _n_categories: usize,
    _categories: *const fmi3String,
) -> fmi3Status {
    fmi3Status::fmi3Error // Not implemented yet
}

#[no_mangle]
pub extern "C" fn fmi3InstantiateCoSimulation(
    _instance_name: fmi3String,
    _instantiation_token: fmi3String,
    _resource_path: fmi3String,
    _visible: fmi3Boolean,
    _logging_on: fmi3Boolean,
    _event_mode_used: fmi3Boolean,
    _early_return_allowed: fmi3Boolean,
    _required_intermediate_variables: *const fmi3ValueReference,
    _n_required_intermediate_variables: usize,
    _instance_environment: fmi3InstanceEnvironment,
    _log_message: fmi3LogMessageCallback,
    _intermediate_update: fmi3IntermediateUpdateCallback,
) -> fmi3Instance {
    std::ptr::null_mut() // Not implemented yet
}

#[no_mangle]
pub extern "C" fn fmi3FreeInstance(_instance: fmi3Instance) {
    // Not implemented yet
}

// Additional stubs will be added as we implement them
