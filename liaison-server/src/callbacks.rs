// FMI 3.0 callback functions for the liaison server
// Implements callbacks that FMUs invoke during operation

// Allow non-standard naming for FMI compatibility
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use crate::fmu_loader::fmi3Status;
use crate::proto::Status as ProtoStatus;
use std::ffi::{CStr, c_char, c_void};
use tracing::{error, info, warn};

// FMI 3.0 types (matching liaison-fmi definitions)
type fmi3InstanceEnvironment = *mut c_void;
type fmi3Char = c_char;
type fmi3String = *const fmi3Char;

/// Context passed to FMI callbacks containing necessary state
///
/// This structure holds references needed by callback functions to:
/// - Publish log messages to the Zenoh network
/// - Track instance-specific state
pub struct CallbackContext {
    /// TODO: Add Zenoh publisher for log messages once server implementation is complete
    /// This will be used to publish log messages received from the FMU to interested clients
    _log_publisher: (),
}

impl CallbackContext {
    /// Create a new callback context
    ///
    /// # Arguments
    /// * `log_publisher` - TODO: Will be a Zenoh publisher for log messages
    ///
    /// # Returns
    /// A new CallbackContext instance
    pub fn new() -> Self {
        CallbackContext {
            _log_publisher: (),
        }
    }
}

impl Default for CallbackContext {
    fn default() -> Self {
        Self::new()
    }
}

/// FMI 3.0 logMessage callback function
///
/// This callback is invoked by the FMU when it needs to log a message.
/// The function:
/// 1. Logs the message using the tracing crate (at appropriate level based on status)
/// 2. Publishes the log message to Zenoh for remote monitoring
///
/// # Safety
/// This is an `extern "C"` callback that will be called by FMU C code.
/// - The `instanceEnvironment` must be a valid pointer to a CallbackContext or null
/// - The `category` and `message` strings must be valid null-terminated C strings or null
/// - Care must be taken when dereferencing any pointers
///
/// # Arguments
/// * `instance_environment` - Pointer to CallbackContext containing Zenoh publisher and state
/// * `status` - The FMI status level for this log message
/// * `category` - Category/source of the log message (e.g., "logAll", "logError")
/// * `message` - The actual log message text
#[no_mangle]
pub unsafe extern "C" fn fmi3_log_message(
    instance_environment: fmi3InstanceEnvironment,
    status: fmi3Status,
    category: fmi3String,
    message: fmi3String,
) {
    // Safely convert C strings to Rust strings
    let category_str = if category.is_null() {
        String::from("(null)")
    } else {
        CStr::from_ptr(category)
            .to_string_lossy()
            .into_owned()
    };

    let message_str = if message.is_null() {
        String::from("(null)")
    } else {
        CStr::from_ptr(message)
            .to_string_lossy()
            .into_owned()
    };

    // Log the message using tracing at the appropriate level
    match status {
        fmi3Status::fmi3OK => {
            info!("[FMU log] fmi3OK: {} : {}", category_str, message_str);
        }
        fmi3Status::fmi3Warning => {
            warn!("[FMU log] fmi3Warning: {} : {}", category_str, message_str);
        }
        fmi3Status::fmi3Discard => {
            warn!("[FMU log] fmi3Discard: {} : {}", category_str, message_str);
        }
        fmi3Status::fmi3Error => {
            error!("[FMU log] fmi3Error: {} : {}", category_str, message_str);
        }
        fmi3Status::fmi3Fatal => {
            error!("[FMU log] fmi3Fatal: {} : {}", category_str, message_str);
        }
    }

    // Publish log message to Zenoh
    // TODO: Implement Zenoh publishing once server is integrated
    // The implementation should:
    // 1. Get the CallbackContext from instance_environment if not null
    // 2. Create a proto::LogMessage with status, category, and message
    // 3. Serialize the message and publish it via Zenoh publisher
    //
    // Example pseudo-code:
    // if !instance_environment.is_null() {
    //     let ctx = unsafe { &*(instance_environment as *const CallbackContext) };
    //     let log_msg = proto::LogMessage {
    //         status: ProtoStatus::from(status) as i32,
    //         category: category_str,
    //         message: message_str,
    //     };
    //     // Serialize and publish via ctx.log_publisher
    // }

    // Suppress unused variable warning for now
    let _ = instance_environment;
}

/// Helper function to convert fmi3Status to protobuf Status
///
/// This mirrors the C++ `transformToProtoStatus` function from liaison.cpp
///
/// # Arguments
/// * `status` - The FMI status to convert
///
/// # Returns
/// The corresponding protobuf Status enum value
pub fn status_to_proto(status: fmi3Status) -> ProtoStatus {
    match status {
        fmi3Status::fmi3OK => ProtoStatus::Ok,
        fmi3Status::fmi3Warning => ProtoStatus::Warning,
        fmi3Status::fmi3Discard => ProtoStatus::Discard,
        fmi3Status::fmi3Error => ProtoStatus::Error,
        fmi3Status::fmi3Fatal => ProtoStatus::Fatal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    // Tests for CallbackContext

    #[test]
    fn test_callback_context_creation() {
        let ctx = CallbackContext::new();
        // Just verify we can create the context
        let _ = ctx;
    }

    #[test]
    fn test_callback_context_default() {
        let ctx = CallbackContext::default();
        let _ = ctx;
    }

    // Tests for status_to_proto conversion function

    #[test]
    fn test_status_to_proto_ok() {
        assert_eq!(status_to_proto(fmi3Status::fmi3OK), ProtoStatus::Ok);
    }

    #[test]
    fn test_status_to_proto_warning() {
        assert_eq!(status_to_proto(fmi3Status::fmi3Warning), ProtoStatus::Warning);
    }

    #[test]
    fn test_status_to_proto_discard() {
        assert_eq!(status_to_proto(fmi3Status::fmi3Discard), ProtoStatus::Discard);
    }

    #[test]
    fn test_status_to_proto_error() {
        assert_eq!(status_to_proto(fmi3Status::fmi3Error), ProtoStatus::Error);
    }

    #[test]
    fn test_status_to_proto_fatal() {
        assert_eq!(status_to_proto(fmi3Status::fmi3Fatal), ProtoStatus::Fatal);
    }

    #[test]
    fn test_status_to_proto_all_variants() {
        // Comprehensive test covering all status variants
        let test_cases = vec![
            (fmi3Status::fmi3OK, ProtoStatus::Ok),
            (fmi3Status::fmi3Warning, ProtoStatus::Warning),
            (fmi3Status::fmi3Discard, ProtoStatus::Discard),
            (fmi3Status::fmi3Error, ProtoStatus::Error),
            (fmi3Status::fmi3Fatal, ProtoStatus::Fatal),
        ];

        for (fmi_status, expected_proto) in test_cases {
            assert_eq!(
                status_to_proto(fmi_status),
                expected_proto,
                "Failed to convert {:?} to {:?}",
                fmi_status,
                expected_proto
            );
        }
    }

    // Tests for fmi3_log_message callback

    #[test]
    fn test_fmi3_log_message_with_valid_strings() {
        // Create test C strings
        let category = CString::new("test_category").unwrap();
        let message = CString::new("test message").unwrap();

        // Call the callback with null context (safe for testing since we don't use it yet)
        fmi3_log_message(
            std::ptr::null_mut(),
            fmi3Status::fmi3OK,
            category.as_ptr(),
            message.as_ptr(),
        );

        // If we get here without crashing, the test passes
    }

    #[test]
    fn test_fmi3_log_message_with_null_category() {
        // Test handling of null category string
        let message = CString::new("test message").unwrap();

        fmi3_log_message(
            std::ptr::null_mut(),
            fmi3Status::fmi3OK,
            std::ptr::null(),
            message.as_ptr(),
        );

        // Should handle null category gracefully
    }

    #[test]
    fn test_fmi3_log_message_with_null_message() {
        // Test handling of null message string
        let category = CString::new("test_category").unwrap();

        fmi3_log_message(
            std::ptr::null_mut(),
            fmi3Status::fmi3OK,
            category.as_ptr(),
            std::ptr::null(),
        );

        // Should handle null message gracefully
    }

    #[test]
    fn test_fmi3_log_message_with_null_strings() {
        // Call with null strings - should handle gracefully
        fmi3_log_message(
            std::ptr::null_mut(),
            fmi3Status::fmi3Warning,
            std::ptr::null(),
            std::ptr::null(),
        );

        // If we get here without crashing, the test passes
    }

    #[test]
    fn test_fmi3_log_message_status_ok() {
        let category = CString::new("logAll").unwrap();
        let message = CString::new("FMU initialized successfully").unwrap();

        fmi3_log_message(
            std::ptr::null_mut(),
            fmi3Status::fmi3OK,
            category.as_ptr(),
            message.as_ptr(),
        );
    }

    #[test]
    fn test_fmi3_log_message_status_warning() {
        let category = CString::new("logWarning").unwrap();
        let message = CString::new("Parameter out of range, using default").unwrap();

        fmi3_log_message(
            std::ptr::null_mut(),
            fmi3Status::fmi3Warning,
            category.as_ptr(),
            message.as_ptr(),
        );
    }

    #[test]
    fn test_fmi3_log_message_status_discard() {
        let category = CString::new("logDiscard").unwrap();
        let message = CString::new("Event discarded").unwrap();

        fmi3_log_message(
            std::ptr::null_mut(),
            fmi3Status::fmi3Discard,
            category.as_ptr(),
            message.as_ptr(),
        );
    }

    #[test]
    fn test_fmi3_log_message_status_error() {
        let category = CString::new("logError").unwrap();
        let message = CString::new("Invalid state transition").unwrap();

        fmi3_log_message(
            std::ptr::null_mut(),
            fmi3Status::fmi3Error,
            category.as_ptr(),
            message.as_ptr(),
        );
    }

    #[test]
    fn test_fmi3_log_message_status_fatal() {
        let category = CString::new("logFatal").unwrap();
        let message = CString::new("Memory allocation failed").unwrap();

        fmi3_log_message(
            std::ptr::null_mut(),
            fmi3Status::fmi3Fatal,
            category.as_ptr(),
            message.as_ptr(),
        );
    }

    #[test]
    fn test_fmi3_log_message_all_status_levels() {
        let category = CString::new("test").unwrap();
        let message = CString::new("message").unwrap();

        // Test all status levels to ensure they all work
        for status in &[
            fmi3Status::fmi3OK,
            fmi3Status::fmi3Warning,
            fmi3Status::fmi3Discard,
            fmi3Status::fmi3Error,
            fmi3Status::fmi3Fatal,
        ] {
            fmi3_log_message(
                std::ptr::null_mut(),
                *status,
                category.as_ptr(),
                message.as_ptr(),
            );
        }
    }

    // Tests for C string handling and edge cases

    #[test]
    fn test_fmi3_log_message_empty_strings() {
        // Test with empty (but valid) C strings
        let category = CString::new("").unwrap();
        let message = CString::new("").unwrap();

        fmi3_log_message(
            std::ptr::null_mut(),
            fmi3Status::fmi3OK,
            category.as_ptr(),
            message.as_ptr(),
        );
    }

    #[test]
    fn test_fmi3_log_message_special_characters() {
        // Test with special characters that might cause issues
        let category = CString::new("log@#$%").unwrap();
        let message = CString::new("Test with special: !@#$%^&*()").unwrap();

        fmi3_log_message(
            std::ptr::null_mut(),
            fmi3Status::fmi3OK,
            category.as_ptr(),
            message.as_ptr(),
        );
    }

    #[test]
    fn test_fmi3_log_message_unicode_characters() {
        // Test with Unicode characters
        let category = CString::new("logUnicode").unwrap();
        let message = CString::new("Test with unicode: αβγδ 你好").unwrap();

        fmi3_log_message(
            std::ptr::null_mut(),
            fmi3Status::fmi3OK,
            category.as_ptr(),
            message.as_ptr(),
        );
    }

    #[test]
    fn test_fmi3_log_message_long_strings() {
        // Test with very long strings
        let long_category = "a".repeat(1000);
        let long_message = "b".repeat(10000);

        let category = CString::new(long_category).unwrap();
        let message = CString::new(long_message).unwrap();

        fmi3_log_message(
            std::ptr::null_mut(),
            fmi3Status::fmi3OK,
            category.as_ptr(),
            message.as_ptr(),
        );
    }

    #[test]
    fn test_fmi3_log_message_with_newlines() {
        // Test with strings containing newlines
        let category = CString::new("logMultiline").unwrap();
        let message = CString::new("Line 1\nLine 2\nLine 3").unwrap();

        fmi3_log_message(
            std::ptr::null_mut(),
            fmi3Status::fmi3Warning,
            category.as_ptr(),
            message.as_ptr(),
        );
    }

    #[test]
    fn test_fmi3_log_message_with_tabs() {
        // Test with strings containing tabs
        let category = CString::new("logTabbed").unwrap();
        let message = CString::new("Col1\tCol2\tCol3").unwrap();

        fmi3_log_message(
            std::ptr::null_mut(),
            fmi3Status::fmi3OK,
            category.as_ptr(),
            message.as_ptr(),
        );
    }

    #[test]
    fn test_fmi3_log_message_with_callback_context() {
        // Test with a valid callback context pointer
        let ctx = CallbackContext::new();
        let ctx_ptr = &ctx as *const CallbackContext as *mut c_void;

        let category = CString::new("test_category").unwrap();
        let message = CString::new("test message").unwrap();

        fmi3_log_message(
            ctx_ptr,
            fmi3Status::fmi3OK,
            category.as_ptr(),
            message.as_ptr(),
        );

        // Currently the callback doesn't use the context, but this tests
        // that passing a valid pointer doesn't cause issues
    }

    #[test]
    fn test_fmi3_log_message_realistic_fmu_messages() {
        // Test with realistic FMU log message patterns
        let test_cases = vec![
            ("logStatusOK", "fmi3EnterInitializationMode: succeeded"),
            ("logStatusWarning", "Time step too large, reducing to 0.001"),
            ("logEvent", "Event detected at time 1.234"),
            ("logSolver", "Newton iteration converged after 5 steps"),
            ("logAll", "Variable 'speed' changed from 0.0 to 5.5"),
        ];

        for (category, msg) in test_cases {
            let category_c = CString::new(category).unwrap();
            let message_c = CString::new(msg).unwrap();

            fmi3_log_message(
                std::ptr::null_mut(),
                fmi3Status::fmi3OK,
                category_c.as_ptr(),
                message_c.as_ptr(),
            );
        }
    }

    #[test]
    fn test_fmi3_log_message_error_scenarios() {
        // Test various error scenarios that an FMU might report
        let error_messages = vec![
            "Division by zero detected",
            "Matrix is singular, cannot invert",
            "Maximum iteration count exceeded",
            "Integration failed to converge",
            "Variable reference 123 not found",
        ];

        for msg in error_messages {
            let category = CString::new("logError").unwrap();
            let message = CString::new(msg).unwrap();

            fmi3_log_message(
                std::ptr::null_mut(),
                fmi3Status::fmi3Error,
                category.as_ptr(),
                message.as_ptr(),
            );
        }
    }

    // Test combinations of null pointers and various status levels

    #[test]
    fn test_fmi3_log_message_null_combinations() {
        // Test all combinations of null/non-null string pointers
        let category = CString::new("test").unwrap();
        let message = CString::new("test").unwrap();

        // Both null
        fmi3_log_message(
            std::ptr::null_mut(),
            fmi3Status::fmi3OK,
            std::ptr::null(),
            std::ptr::null(),
        );

        // Category null, message valid
        fmi3_log_message(
            std::ptr::null_mut(),
            fmi3Status::fmi3Warning,
            std::ptr::null(),
            message.as_ptr(),
        );

        // Category valid, message null
        fmi3_log_message(
            std::ptr::null_mut(),
            fmi3Status::fmi3Error,
            category.as_ptr(),
            std::ptr::null(),
        );

        // Both valid
        fmi3_log_message(
            std::ptr::null_mut(),
            fmi3Status::fmi3Fatal,
            category.as_ptr(),
            message.as_ptr(),
        );
    }
}
