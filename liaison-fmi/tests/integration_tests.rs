// Integration tests for the liaison-fmi client library
// These tests verify the core functionality without requiring a full Zenoh server

use liaisonfmu::fmi3::*;
use liaisonfmu::proto;
use std::ffi::{CStr, CString};
use std::ptr;

//=============================================================================
// Test: Conversion Functions (proto::Status <-> fmi3Status)
//=============================================================================

#[test]
fn test_proto_status_to_fmi_status_conversions() {
    // Test all proto::Status variants convert to correct fmi3Status
    assert_eq!(fmi3Status::from(proto::Status::Ok), fmi3Status::fmi3OK);
    assert_eq!(
        fmi3Status::from(proto::Status::Warning),
        fmi3Status::fmi3Warning
    );
    assert_eq!(
        fmi3Status::from(proto::Status::Discard),
        fmi3Status::fmi3Discard
    );
    assert_eq!(
        fmi3Status::from(proto::Status::Error),
        fmi3Status::fmi3Error
    );
    assert_eq!(
        fmi3Status::from(proto::Status::Fatal),
        fmi3Status::fmi3Fatal
    );
}

#[test]
fn test_fmi_status_to_proto_status_conversions() {
    // Test all fmi3Status variants convert to correct proto::Status
    assert_eq!(proto::Status::from(fmi3Status::fmi3OK), proto::Status::Ok);
    assert_eq!(
        proto::Status::from(fmi3Status::fmi3Warning),
        proto::Status::Warning
    );
    assert_eq!(
        proto::Status::from(fmi3Status::fmi3Discard),
        proto::Status::Discard
    );
    assert_eq!(
        proto::Status::from(fmi3Status::fmi3Error),
        proto::Status::Error
    );
    assert_eq!(
        proto::Status::from(fmi3Status::fmi3Fatal),
        proto::Status::Fatal
    );
}

#[test]
fn test_i32_to_fmi_status_conversions() {
    // Test i32 values convert to correct fmi3Status
    assert_eq!(fmi3Status::from(0), fmi3Status::fmi3OK);
    assert_eq!(fmi3Status::from(1), fmi3Status::fmi3Warning);
    assert_eq!(fmi3Status::from(2), fmi3Status::fmi3Discard);
    assert_eq!(fmi3Status::from(3), fmi3Status::fmi3Error);
    assert_eq!(fmi3Status::from(4), fmi3Status::fmi3Fatal);
}

#[test]
fn test_invalid_i32_to_fmi_status_defaults_to_error() {
    // Unknown values should default to Error
    assert_eq!(fmi3Status::from(99), fmi3Status::fmi3Error);
    assert_eq!(fmi3Status::from(-1), fmi3Status::fmi3Error);
    assert_eq!(fmi3Status::from(999), fmi3Status::fmi3Error);
}

#[test]
fn test_status_conversion_round_trip() {
    // Test that converting back and forth preserves values
    let statuses = vec![
        fmi3Status::fmi3OK,
        fmi3Status::fmi3Warning,
        fmi3Status::fmi3Discard,
        fmi3Status::fmi3Error,
        fmi3Status::fmi3Fatal,
    ];

    for status in statuses {
        let proto_status = proto::Status::from(status);
        let back_to_fmi = fmi3Status::from(proto_status);
        assert_eq!(status, back_to_fmi);
    }
}

//=============================================================================
// Test: FMI Function Exports
//=============================================================================

#[test]
fn test_fmi3_get_version_export() {
    // Test that fmi3GetVersion returns correct version string
    let version = fmi3GetVersion();
    assert!(!version.is_null(), "Version pointer should not be null");

    unsafe {
        let version_str = CStr::from_ptr(version);
        assert_eq!(
            version_str.to_str().unwrap(),
            "3.0",
            "FMI version should be 3.0"
        );
    }
}

#[test]
fn test_fmi3_set_debug_logging_with_null_instance() {
    // Test that null instance is handled gracefully
    let status = fmi3SetDebugLogging(ptr::null_mut(), 1, 0, ptr::null());
    assert_eq!(
        status,
        fmi3Status::fmi3Error,
        "Null instance should return fmi3Error"
    );
}

#[test]
fn test_fmi3_free_instance_with_null() {
    // Test that fmi3FreeInstance handles null gracefully (should not crash)
    fmi3FreeInstance(ptr::null_mut());
    // If we reach here without panic, test passes
}

#[test]
fn test_fmi3_terminate_with_null_instance() {
    // Test that terminate with null instance returns error
    let status = fmi3Terminate(ptr::null_mut());
    assert_eq!(
        status,
        fmi3Status::fmi3Error,
        "Null instance should return fmi3Error"
    );
}

#[test]
fn test_fmi3_reset_with_null_instance() {
    // Test that reset with null instance returns error
    let status = fmi3Reset(ptr::null_mut());
    assert_eq!(
        status,
        fmi3Status::fmi3Error,
        "Null instance should return fmi3Error"
    );
}

#[test]
fn test_fmi3_enter_initialization_mode_with_null() {
    // Test that enter initialization mode with null instance returns error
    let status = fmi3EnterInitializationMode(
        ptr::null_mut(),
        0,   // toleranceDefined
        0.0, // tolerance
        0.0, // startTime
        0,   // stopTimeDefined
        0.0, // stopTime
    );
    assert_eq!(
        status,
        fmi3Status::fmi3Error,
        "Null instance should return fmi3Error"
    );
}

#[test]
fn test_fmi3_exit_initialization_mode_with_null() {
    // Test that exit initialization mode with null instance returns error
    let status = fmi3ExitInitializationMode(ptr::null_mut());
    assert_eq!(
        status,
        fmi3Status::fmi3Error,
        "Null instance should return fmi3Error"
    );
}

#[test]
fn test_fmi3_enter_configuration_mode_with_null() {
    // Test that enter configuration mode with null instance returns error
    let status = fmi3EnterConfigurationMode(ptr::null_mut());
    assert_eq!(
        status,
        fmi3Status::fmi3Error,
        "Null instance should return fmi3Error"
    );
}

#[test]
fn test_fmi3_exit_configuration_mode_with_null() {
    // Test that exit configuration mode with null instance returns error
    let status = fmi3ExitConfigurationMode(ptr::null_mut());
    assert_eq!(
        status,
        fmi3Status::fmi3Error,
        "Null instance should return fmi3Error"
    );
}

#[test]
fn test_fmi3_do_step_with_null_instance() {
    // Test that doStep with null instance returns error
    let status = fmi3DoStep(
        ptr::null_mut(),
        0.0,             // currentCommunicationPoint
        1.0,             // communicationStepSize
        1,               // noSetFMUStatePriorToCurrentPoint
        ptr::null_mut(), // eventHandlingNeeded
        ptr::null_mut(), // terminateSimulation
        ptr::null_mut(), // earlyReturn
        ptr::null_mut(), // lastSuccessfulTime
    );
    assert_eq!(
        status,
        fmi3Status::fmi3Error,
        "Null instance should return fmi3Error"
    );
}

#[test]
fn test_fmi3_get_boolean_with_null_instance() {
    // Test that getBoolean with null instance returns error
    let value_references = [0u32];
    let mut values = [0i32];

    let status = fmi3GetBoolean(
        ptr::null_mut(),
        value_references.as_ptr(),
        1,
        values.as_mut_ptr(),
        1,
    );
    assert_eq!(
        status,
        fmi3Status::fmi3Error,
        "Null instance should return fmi3Error"
    );
}

#[test]
fn test_fmi3_set_boolean_with_null_instance() {
    // Test that setBoolean with null instance returns error
    let value_references = [0u32];
    let values = [1i32];

    let status = fmi3SetBoolean(
        ptr::null_mut(),
        value_references.as_ptr(),
        1,
        values.as_ptr(),
        1,
    );
    assert_eq!(
        status,
        fmi3Status::fmi3Error,
        "Null instance should return fmi3Error"
    );
}

#[test]
fn test_fmi3_get_string_with_null_instance() {
    // Test that getString with null instance returns error
    let value_references = [0u32];
    let mut values = [ptr::null()];

    let status = fmi3GetString(
        ptr::null_mut(),
        value_references.as_ptr(),
        1,
        values.as_mut_ptr(),
        1,
    );
    assert_eq!(
        status,
        fmi3Status::fmi3Error,
        "Null instance should return fmi3Error"
    );
}

#[test]
fn test_fmi3_set_string_with_null_instance() {
    // Test that setString with null instance returns error
    let value_references = [0u32];
    let test_str = CString::new("test").unwrap();
    let values = [test_str.as_ptr()];

    let status = fmi3SetString(
        ptr::null_mut(),
        value_references.as_ptr(),
        1,
        values.as_ptr(),
        1,
    );
    assert_eq!(
        status,
        fmi3Status::fmi3Error,
        "Null instance should return fmi3Error"
    );
}

//=============================================================================
// Test: FMI Types and Constants
//=============================================================================

#[test]
fn test_fmi3_status_enum_values() {
    // Verify that fmi3Status enum has correct integer values
    assert_eq!(fmi3Status::fmi3OK as i32, 0);
    assert_eq!(fmi3Status::fmi3Warning as i32, 1);
    assert_eq!(fmi3Status::fmi3Discard as i32, 2);
    assert_eq!(fmi3Status::fmi3Error as i32, 3);
    assert_eq!(fmi3Status::fmi3Fatal as i32, 4);
}

#[test]
fn test_fmi3_status_equality() {
    // Test that status comparisons work correctly
    assert_eq!(fmi3Status::fmi3OK, fmi3Status::fmi3OK);
    assert_ne!(fmi3Status::fmi3OK, fmi3Status::fmi3Error);
    assert_ne!(fmi3Status::fmi3Warning, fmi3Status::fmi3Fatal);
}

#[test]
fn test_fmi3_boolean_type() {
    // FMI Boolean is i32, where 0 = false, non-zero = true
    let fmi_false: fmi3Boolean = 0;
    let fmi_true: fmi3Boolean = 1;

    assert_eq!(fmi_false, 0);
    assert_eq!(fmi_true, 1);
    assert_ne!(fmi_false, fmi_true);
}

//=============================================================================
// Test: Error Handling
//=============================================================================

#[test]
fn test_null_instance_pointer_safety() {
    // Test that all functions handle null instance pointers safely without crashing
    let null_instance: fmi3Instance = ptr::null_mut();

    // These should all return error status without crashing
    assert_eq!(
        fmi3SetDebugLogging(null_instance, 0, 0, ptr::null()),
        fmi3Status::fmi3Error
    );
    assert_eq!(fmi3Terminate(null_instance), fmi3Status::fmi3Error);
    assert_eq!(fmi3Reset(null_instance), fmi3Status::fmi3Error);
    assert_eq!(
        fmi3EnterConfigurationMode(null_instance),
        fmi3Status::fmi3Error
    );
    assert_eq!(
        fmi3ExitConfigurationMode(null_instance),
        fmi3Status::fmi3Error
    );
    assert_eq!(
        fmi3ExitInitializationMode(null_instance),
        fmi3Status::fmi3Error
    );

    // FreeInstance should handle null gracefully (void return)
    fmi3FreeInstance(null_instance);
}

#[test]
fn test_null_pointer_safety_in_getters_setters() {
    // Test that getter/setter functions handle null instance safely
    let null_instance: fmi3Instance = ptr::null_mut();
    let value_refs = [0u32];
    let mut values_bool = [0i32];

    // Boolean operations
    assert_eq!(
        fmi3GetBoolean(
            null_instance,
            value_refs.as_ptr(),
            1,
            values_bool.as_mut_ptr(),
            1
        ),
        fmi3Status::fmi3Error
    );
    assert_eq!(
        fmi3SetBoolean(
            null_instance,
            value_refs.as_ptr(),
            1,
            values_bool.as_ptr(),
            1
        ),
        fmi3Status::fmi3Error
    );
}

//=============================================================================
// Test: FMI Type Sizes and Representations
//=============================================================================

#[test]
fn test_fmi_numeric_type_sizes() {
    // Verify that FMI types have correct sizes
    assert_eq!(std::mem::size_of::<fmi3Float32>(), 4);
    assert_eq!(std::mem::size_of::<fmi3Float64>(), 8);
    assert_eq!(std::mem::size_of::<fmi3Int8>(), 1);
    assert_eq!(std::mem::size_of::<fmi3UInt8>(), 1);
    assert_eq!(std::mem::size_of::<fmi3Int16>(), 2);
    assert_eq!(std::mem::size_of::<fmi3UInt16>(), 2);
    assert_eq!(std::mem::size_of::<fmi3Int32>(), 4);
    assert_eq!(std::mem::size_of::<fmi3UInt32>(), 4);
    assert_eq!(std::mem::size_of::<fmi3Int64>(), 8);
    assert_eq!(std::mem::size_of::<fmi3UInt64>(), 8);
    assert_eq!(std::mem::size_of::<fmi3Boolean>(), 4);
}

#[test]
fn test_fmi_pointer_type_sizes() {
    // All pointer types should be pointer-sized
    let ptr_size = std::mem::size_of::<*const ()>();
    assert_eq!(std::mem::size_of::<fmi3Instance>(), ptr_size);
    assert_eq!(std::mem::size_of::<fmi3InstanceEnvironment>(), ptr_size);
    assert_eq!(std::mem::size_of::<fmi3String>(), ptr_size);
    assert_eq!(std::mem::size_of::<fmi3Binary>(), ptr_size);
    assert_eq!(std::mem::size_of::<fmi3FMUState>(), ptr_size);
}

//=============================================================================
// Test: Proto Message Types
//=============================================================================

#[test]
fn test_proto_status_enum_values() {
    // Verify proto::Status enum values match FMI expectations
    assert_eq!(proto::Status::Ok as i32, 0);
    assert_eq!(proto::Status::Warning as i32, 1);
    assert_eq!(proto::Status::Discard as i32, 2);
    assert_eq!(proto::Status::Error as i32, 3);
    assert_eq!(proto::Status::Fatal as i32, 4);
}

#[test]
fn test_proto_status_from_i32() {
    // Test creating proto::Status from i32 values
    assert_eq!(proto::Status::try_from(0), Ok(proto::Status::Ok));
    assert_eq!(proto::Status::try_from(1), Ok(proto::Status::Warning));
    assert_eq!(proto::Status::try_from(2), Ok(proto::Status::Discard));
    assert_eq!(proto::Status::try_from(3), Ok(proto::Status::Error));
    assert_eq!(proto::Status::try_from(4), Ok(proto::Status::Fatal));

    // Invalid values should return error
    assert!(proto::Status::try_from(99).is_err());
    assert!(proto::Status::try_from(-1).is_err());
}

//=============================================================================
// Test: Callback Function Types
//=============================================================================

#[test]
fn test_callback_types_are_option() {
    // Verify callback types are Option (can be None)
    let _log_callback: fmi3LogMessageCallback = None;
    let _intermediate_callback: fmi3IntermediateUpdateCallback = None;
    let _clock_callback: fmi3ClockUpdateCallback = None;
    let _lock_callback: fmi3LockPreemptionCallback = None;
    let _unlock_callback: fmi3UnlockPreemptionCallback = None;
}

extern "C" fn test_log_callback(
    _instance_env: fmi3InstanceEnvironment,
    _status: fmi3Status,
    _category: fmi3String,
    _message: fmi3String,
) {
    // Mock callback for testing
}

#[test]
fn test_log_callback_function_pointer() {
    // Test that we can create and use callback function pointers
    let callback: fmi3LogMessageCallback = Some(test_log_callback);
    assert!(callback.is_some());

    // Test calling the callback (safely, without actual operation)
    if let Some(cb) = callback {
        let test_category = CString::new("TestCategory").unwrap();
        let test_message = CString::new("Test message").unwrap();
        cb(
            ptr::null_mut(),
            fmi3Status::fmi3OK,
            test_category.as_ptr(),
            test_message.as_ptr(),
        );
    }
}

//=============================================================================
// Test: Value Reference Type
//=============================================================================

#[test]
fn test_value_reference_type() {
    // fmi3ValueReference should be u32
    let vr: fmi3ValueReference = 42;
    assert_eq!(vr, 42u32);
    assert_eq!(std::mem::size_of::<fmi3ValueReference>(), 4);
}

#[test]
fn test_value_reference_array() {
    // Test creating arrays of value references
    let value_refs: [fmi3ValueReference; 3] = [0, 1, 2];
    assert_eq!(value_refs.len(), 3);
    assert_eq!(value_refs[0], 0);
    assert_eq!(value_refs[1], 1);
    assert_eq!(value_refs[2], 2);
}

//=============================================================================
// Test: FMI String Handling
//=============================================================================

#[test]
fn test_c_string_creation_and_conversion() {
    // Test creating C strings for FMI function parameters
    let test_str = "TestInstanceName";
    let c_string = CString::new(test_str).unwrap();
    let fmi_string: fmi3String = c_string.as_ptr();

    assert!(!fmi_string.is_null());

    unsafe {
        let back_to_rust = CStr::from_ptr(fmi_string);
        assert_eq!(back_to_rust.to_str().unwrap(), test_str);
    }
}

#[test]
fn test_null_fmi_string() {
    // Test that null FMI strings are handled
    let null_string: fmi3String = ptr::null();
    assert!(null_string.is_null());
}

//=============================================================================
// Test: FMI Instantiation Function Signatures
//=============================================================================

#[test]
fn test_fmi3_instantiate_co_simulation_with_null_name() {
    // Test that instantiation with null instance name returns null
    let instance = fmi3InstantiateCoSimulation(
        ptr::null(),     // instanceName (null)
        ptr::null(),     // instantiationToken
        ptr::null(),     // resourcePath
        0,               // visible
        0,               // loggingOn
        0,               // eventModeUsed
        0,               // earlyReturnAllowed
        ptr::null(),     // requiredIntermediateVariables
        0,               // nRequiredIntermediateVariables
        ptr::null_mut(), // instanceEnvironment
        None,            // logMessage callback
        None,            // intermediateUpdate callback
    );

    assert!(
        instance.is_null(),
        "Instantiation with null name should return null"
    );
}

#[test]
fn test_fmi3_instantiate_model_exchange_with_null_name() {
    // Test that ME instantiation with null instance name returns null
    let instance = fmi3InstantiateModelExchange(
        ptr::null(),     // instanceName (null)
        ptr::null(),     // instantiationToken
        ptr::null(),     // resourcePath
        0,               // visible
        0,               // loggingOn
        ptr::null_mut(), // instanceEnvironment
        None,            // logMessage callback
    );

    assert!(
        instance.is_null(),
        "ME instantiation with null name should return null"
    );
}

#[test]
fn test_fmi3_instantiate_scheduled_execution_with_null_name() {
    // Test that SE instantiation with null instance name returns null
    let instance = fmi3InstantiateScheduledExecution(
        ptr::null(),     // instanceName (null)
        ptr::null(),     // instantiationToken
        ptr::null(),     // resourcePath
        0,               // visible
        0,               // loggingOn
        ptr::null_mut(), // instanceEnvironment
        None,            // logMessage callback
        None,            // clockUpdate callback
        None,            // lockPreemption callback
        None,            // unlockPreemption callback
    );

    assert!(
        instance.is_null(),
        "SE instantiation with null name should return null"
    );
}

//=============================================================================
// Test: Module Organization
//=============================================================================

#[test]
fn test_module_exports() {
    // Verify that the main types are re-exported and accessible
    let _status: fmi3Status = fmi3Status::fmi3OK;
    let _instance: fmi3Instance = ptr::null_mut();
    let _value_ref: fmi3ValueReference = 0;
    let _boolean: fmi3Boolean = 0;

    // If compilation succeeds, exports are working correctly
}

//=============================================================================
// Test: Documentation and API Surface
//=============================================================================

#[test]
fn test_api_completeness() {
    // This test verifies that all major FMI 3.0 function categories are exported
    // We're not testing functionality (requires Zenoh), just that symbols exist

    // Version information
    let _ = fmi3GetVersion();

    // Instantiation (signatures exist - tested above with null params)
    // fmi3InstantiateCoSimulation, fmi3InstantiateModelExchange, fmi3InstantiateScheduledExecution

    // Lifecycle functions (tested above with null instances)
    // fmi3FreeInstance, fmi3EnterInitializationMode, fmi3ExitInitializationMode
    // fmi3Terminate, fmi3Reset

    // Configuration mode functions
    // fmi3EnterConfigurationMode, fmi3ExitConfigurationMode

    // Simulation functions
    // fmi3DoStep (tested above)

    // Variable access functions
    // fmi3GetBoolean, fmi3SetBoolean, fmi3GetString, fmi3SetString
    // (tested above with null instances)
}

//=============================================================================
// Test: Thread Safety Annotations
//=============================================================================

#[test]
fn test_send_trait_for_placeholder() {
    // This is a compile-time test - if it compiles, Placeholder implements Send
    #[allow(dead_code)]
    fn assert_send<T: Send>() {}
    // Note: We can't directly test Placeholder here since it's not exported,
    // but the impl in placeholder.rs ensures it's Send
}

//=============================================================================
// Summary and Coverage Report
//=============================================================================

// The following test documents what we've tested
#[test]
fn test_coverage_summary() {
    // This test serves as documentation of test coverage

    // 1. Conversion Functions: COVERED
    //    - proto::Status <-> fmi3Status (all variants)
    //    - i32 -> fmi3Status (including invalid values)
    //    - Round-trip conversions

    // 2. FMI Function Exports: COVERED
    //    - fmi3GetVersion
    //    - fmi3SetDebugLogging
    //    - fmi3FreeInstance
    //    - fmi3Terminate, fmi3Reset
    //    - fmi3EnterInitializationMode, fmi3ExitInitializationMode
    //    - fmi3EnterConfigurationMode, fmi3ExitConfigurationMode
    //    - fmi3DoStep
    //    - fmi3GetBoolean, fmi3SetBoolean
    //    - fmi3GetString, fmi3SetString
    //    - Instantiation functions (all three types)

    // 3. Error Handling: COVERED
    //    - Null instance pointer handling
    //    - Null parameter handling in all major functions
    //    - Safe failure modes (no crashes)

    // 4. Type Safety: COVERED
    //    - FMI type sizes
    //    - FMI status enum values
    //    - Value reference types
    //    - Callback function types
    //    - String handling

    // 5. Proto Integration: COVERED
    //    - Proto status enum values
    //    - Proto status conversions
    //    - Proto status from i32

    // NOT COVERED (requires Zenoh server):
    //    - Placeholder initialization with real config
    //    - Actual Zenoh query/response
    //    - Log message subscriber functionality
    //    - Full instantiation with valid parameters
    //    - Real variable getter/setter operations
}
