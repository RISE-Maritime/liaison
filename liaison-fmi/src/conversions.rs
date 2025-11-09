// Type conversion utilities between protobuf types and FMI types

use crate::fmi3::fmi3Status;
use crate::proto::Status as ProtoStatus;

/// Convert from protobuf Status to FMI status
impl From<ProtoStatus> for fmi3Status {
    fn from(proto_status: ProtoStatus) -> Self {
        match proto_status {
            ProtoStatus::Ok => fmi3Status::fmi3OK,
            ProtoStatus::Warning => fmi3Status::fmi3Warning,
            ProtoStatus::Discard => fmi3Status::fmi3Discard,
            ProtoStatus::Error => fmi3Status::fmi3Error,
            ProtoStatus::Fatal => fmi3Status::fmi3Fatal,
        }
    }
}

/// Convert from FMI status to protobuf Status
impl From<fmi3Status> for ProtoStatus {
    fn from(fmi_status: fmi3Status) -> Self {
        match fmi_status {
            fmi3Status::fmi3OK => ProtoStatus::Ok,
            fmi3Status::fmi3Warning => ProtoStatus::Warning,
            fmi3Status::fmi3Discard => ProtoStatus::Discard,
            fmi3Status::fmi3Error => ProtoStatus::Error,
            fmi3Status::fmi3Fatal => ProtoStatus::Fatal,
        }
    }
}

/// Convert from protobuf Status to i32 (for FMI status)
impl From<i32> for fmi3Status {
    fn from(value: i32) -> Self {
        match value {
            0 => fmi3Status::fmi3OK,
            1 => fmi3Status::fmi3Warning,
            2 => fmi3Status::fmi3Discard,
            3 => fmi3Status::fmi3Error,
            4 => fmi3Status::fmi3Fatal,
            _ => fmi3Status::fmi3Error, // Default to error for unknown values
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =============================================================================
    // ProtoStatus -> fmi3Status Conversion Tests
    // =============================================================================

    /// Test conversion of all valid ProtoStatus values to fmi3Status
    ///
    /// This test ensures that each protobuf status value correctly maps to its
    /// corresponding FMI status code according to the FMI 3.0 specification.
    #[test]
    fn test_proto_to_fmi_status_all_values() {
        assert_eq!(fmi3Status::from(ProtoStatus::Ok), fmi3Status::fmi3OK);
        assert_eq!(
            fmi3Status::from(ProtoStatus::Warning),
            fmi3Status::fmi3Warning
        );
        assert_eq!(
            fmi3Status::from(ProtoStatus::Discard),
            fmi3Status::fmi3Discard
        );
        assert_eq!(fmi3Status::from(ProtoStatus::Error), fmi3Status::fmi3Error);
        assert_eq!(fmi3Status::from(ProtoStatus::Fatal), fmi3Status::fmi3Fatal);
    }

    /// Test that ProtoStatus::Ok converts to fmi3OK (value 0)
    #[test]
    fn test_proto_ok_to_fmi() {
        let fmi_status = fmi3Status::from(ProtoStatus::Ok);
        assert_eq!(fmi_status, fmi3Status::fmi3OK);
        assert_eq!(fmi_status as i32, 0);
    }

    /// Test that ProtoStatus::Warning converts to fmi3Warning (value 1)
    #[test]
    fn test_proto_warning_to_fmi() {
        let fmi_status = fmi3Status::from(ProtoStatus::Warning);
        assert_eq!(fmi_status, fmi3Status::fmi3Warning);
        assert_eq!(fmi_status as i32, 1);
    }

    /// Test that ProtoStatus::Discard converts to fmi3Discard (value 2)
    #[test]
    fn test_proto_discard_to_fmi() {
        let fmi_status = fmi3Status::from(ProtoStatus::Discard);
        assert_eq!(fmi_status, fmi3Status::fmi3Discard);
        assert_eq!(fmi_status as i32, 2);
    }

    /// Test that ProtoStatus::Error converts to fmi3Error (value 3)
    #[test]
    fn test_proto_error_to_fmi() {
        let fmi_status = fmi3Status::from(ProtoStatus::Error);
        assert_eq!(fmi_status, fmi3Status::fmi3Error);
        assert_eq!(fmi_status as i32, 3);
    }

    /// Test that ProtoStatus::Fatal converts to fmi3Fatal (value 4)
    #[test]
    fn test_proto_fatal_to_fmi() {
        let fmi_status = fmi3Status::from(ProtoStatus::Fatal);
        assert_eq!(fmi_status, fmi3Status::fmi3Fatal);
        assert_eq!(fmi_status as i32, 4);
    }

    // =============================================================================
    // fmi3Status -> ProtoStatus Conversion Tests
    // =============================================================================

    /// Test conversion of all valid fmi3Status values to ProtoStatus
    ///
    /// This test ensures bidirectional conversion works correctly and that
    /// FMI status codes are properly mapped back to protobuf status values.
    #[test]
    fn test_fmi_to_proto_status_all_values() {
        assert_eq!(ProtoStatus::from(fmi3Status::fmi3OK), ProtoStatus::Ok);
        assert_eq!(
            ProtoStatus::from(fmi3Status::fmi3Warning),
            ProtoStatus::Warning
        );
        assert_eq!(
            ProtoStatus::from(fmi3Status::fmi3Discard),
            ProtoStatus::Discard
        );
        assert_eq!(ProtoStatus::from(fmi3Status::fmi3Error), ProtoStatus::Error);
        assert_eq!(ProtoStatus::from(fmi3Status::fmi3Fatal), ProtoStatus::Fatal);
    }

    /// Test that fmi3OK converts to ProtoStatus::Ok
    #[test]
    fn test_fmi_ok_to_proto() {
        let proto_status = ProtoStatus::from(fmi3Status::fmi3OK);
        assert_eq!(proto_status, ProtoStatus::Ok);
        assert_eq!(proto_status as i32, 0);
    }

    /// Test that fmi3Warning converts to ProtoStatus::Warning
    #[test]
    fn test_fmi_warning_to_proto() {
        let proto_status = ProtoStatus::from(fmi3Status::fmi3Warning);
        assert_eq!(proto_status, ProtoStatus::Warning);
        assert_eq!(proto_status as i32, 1);
    }

    /// Test that fmi3Discard converts to ProtoStatus::Discard
    #[test]
    fn test_fmi_discard_to_proto() {
        let proto_status = ProtoStatus::from(fmi3Status::fmi3Discard);
        assert_eq!(proto_status, ProtoStatus::Discard);
        assert_eq!(proto_status as i32, 2);
    }

    /// Test that fmi3Error converts to ProtoStatus::Error
    #[test]
    fn test_fmi_error_to_proto() {
        let proto_status = ProtoStatus::from(fmi3Status::fmi3Error);
        assert_eq!(proto_status, ProtoStatus::Error);
        assert_eq!(proto_status as i32, 3);
    }

    /// Test that fmi3Fatal converts to ProtoStatus::Fatal
    #[test]
    fn test_fmi_fatal_to_proto() {
        let proto_status = ProtoStatus::from(fmi3Status::fmi3Fatal);
        assert_eq!(proto_status, ProtoStatus::Fatal);
        assert_eq!(proto_status as i32, 4);
    }

    // =============================================================================
    // i32 -> fmi3Status Conversion Tests
    // =============================================================================

    /// Test conversion of all valid i32 values to fmi3Status
    ///
    /// This test verifies that integer values from external sources (e.g., C FFI)
    /// are correctly interpreted as FMI status codes.
    #[test]
    fn test_i32_to_fmi_status_valid_values() {
        assert_eq!(fmi3Status::from(0), fmi3Status::fmi3OK);
        assert_eq!(fmi3Status::from(1), fmi3Status::fmi3Warning);
        assert_eq!(fmi3Status::from(2), fmi3Status::fmi3Discard);
        assert_eq!(fmi3Status::from(3), fmi3Status::fmi3Error);
        assert_eq!(fmi3Status::from(4), fmi3Status::fmi3Fatal);
    }

    /// Test that i32 value 0 converts to fmi3OK
    #[test]
    fn test_i32_zero_to_fmi_ok() {
        assert_eq!(fmi3Status::from(0_i32), fmi3Status::fmi3OK);
    }

    /// Test that i32 value 1 converts to fmi3Warning
    #[test]
    fn test_i32_one_to_fmi_warning() {
        assert_eq!(fmi3Status::from(1_i32), fmi3Status::fmi3Warning);
    }

    /// Test that i32 value 2 converts to fmi3Discard
    #[test]
    fn test_i32_two_to_fmi_discard() {
        assert_eq!(fmi3Status::from(2_i32), fmi3Status::fmi3Discard);
    }

    /// Test that i32 value 3 converts to fmi3Error
    #[test]
    fn test_i32_three_to_fmi_error() {
        assert_eq!(fmi3Status::from(3_i32), fmi3Status::fmi3Error);
    }

    /// Test that i32 value 4 converts to fmi3Fatal
    #[test]
    fn test_i32_four_to_fmi_fatal() {
        assert_eq!(fmi3Status::from(4_i32), fmi3Status::fmi3Fatal);
    }

    // =============================================================================
    // Edge Cases: Invalid i32 Values
    // =============================================================================

    /// Test that invalid i32 values default to fmi3Error
    ///
    /// This is a safety measure to handle corrupted or invalid status codes
    /// from external sources. The implementation defaults to Error status
    /// for any unrecognized value.
    #[test]
    fn test_i32_invalid_values_default_to_error() {
        // Positive out-of-range values
        assert_eq!(fmi3Status::from(5), fmi3Status::fmi3Error);
        assert_eq!(fmi3Status::from(10), fmi3Status::fmi3Error);
        assert_eq!(fmi3Status::from(99), fmi3Status::fmi3Error);
        assert_eq!(fmi3Status::from(100), fmi3Status::fmi3Error);
        assert_eq!(fmi3Status::from(1000), fmi3Status::fmi3Error);
        assert_eq!(fmi3Status::from(i32::MAX), fmi3Status::fmi3Error);
    }

    /// Test that negative i32 values default to fmi3Error
    #[test]
    fn test_i32_negative_values_default_to_error() {
        assert_eq!(fmi3Status::from(-1), fmi3Status::fmi3Error);
        assert_eq!(fmi3Status::from(-5), fmi3Status::fmi3Error);
        assert_eq!(fmi3Status::from(-100), fmi3Status::fmi3Error);
        assert_eq!(fmi3Status::from(i32::MIN), fmi3Status::fmi3Error);
    }

    /// Test extreme edge case values
    #[test]
    fn test_i32_extreme_edge_cases() {
        assert_eq!(fmi3Status::from(i32::MIN), fmi3Status::fmi3Error);
        assert_eq!(fmi3Status::from(i32::MIN + 1), fmi3Status::fmi3Error);
        assert_eq!(fmi3Status::from(i32::MAX - 1), fmi3Status::fmi3Error);
        assert_eq!(fmi3Status::from(i32::MAX), fmi3Status::fmi3Error);
    }

    // =============================================================================
    // Round-trip Conversion Tests
    // =============================================================================

    /// Test that ProtoStatus -> fmi3Status -> ProtoStatus round-trip preserves values
    ///
    /// This ensures that converting between types and back again doesn't lose
    /// information or introduce errors.
    #[test]
    fn test_proto_to_fmi_to_proto_roundtrip() {
        let statuses = vec![
            ProtoStatus::Ok,
            ProtoStatus::Warning,
            ProtoStatus::Discard,
            ProtoStatus::Error,
            ProtoStatus::Fatal,
        ];

        for original_status in statuses {
            let fmi_status = fmi3Status::from(original_status);
            let roundtrip_status = ProtoStatus::from(fmi_status);
            assert_eq!(
                original_status, roundtrip_status,
                "Round-trip conversion failed for {:?}",
                original_status
            );
        }
    }

    /// Test that fmi3Status -> ProtoStatus -> fmi3Status round-trip preserves values
    #[test]
    fn test_fmi_to_proto_to_fmi_roundtrip() {
        let statuses = vec![
            fmi3Status::fmi3OK,
            fmi3Status::fmi3Warning,
            fmi3Status::fmi3Discard,
            fmi3Status::fmi3Error,
            fmi3Status::fmi3Fatal,
        ];

        for original_status in statuses {
            let proto_status = ProtoStatus::from(original_status);
            let roundtrip_status = fmi3Status::from(proto_status);
            assert_eq!(
                original_status, roundtrip_status,
                "Round-trip conversion failed for {:?}",
                original_status
            );
        }
    }

    /// Test that i32 -> fmi3Status -> i32 round-trip preserves valid values
    #[test]
    fn test_i32_to_fmi_to_i32_roundtrip_valid() {
        for i in 0..=4 {
            let fmi_status = fmi3Status::from(i);
            let roundtrip_value = fmi_status as i32;
            assert_eq!(
                i, roundtrip_value,
                "Round-trip conversion failed for i32 value {}",
                i
            );
        }
    }

    // =============================================================================
    // Type Safety and Consistency Tests
    // =============================================================================

    /// Verify that fmi3Status enum values match FMI 3.0 specification
    #[test]
    fn test_fmi_status_numeric_values() {
        assert_eq!(fmi3Status::fmi3OK as i32, 0);
        assert_eq!(fmi3Status::fmi3Warning as i32, 1);
        assert_eq!(fmi3Status::fmi3Discard as i32, 2);
        assert_eq!(fmi3Status::fmi3Error as i32, 3);
        assert_eq!(fmi3Status::fmi3Fatal as i32, 4);
    }

    /// Verify that ProtoStatus enum values match protobuf definition
    #[test]
    fn test_proto_status_numeric_values() {
        assert_eq!(ProtoStatus::Ok as i32, 0);
        assert_eq!(ProtoStatus::Warning as i32, 1);
        assert_eq!(ProtoStatus::Discard as i32, 2);
        assert_eq!(ProtoStatus::Error as i32, 3);
        assert_eq!(ProtoStatus::Fatal as i32, 4);
    }

    /// Verify that fmi3Status implements expected traits
    #[test]
    fn test_fmi_status_traits() {
        let status = fmi3Status::fmi3OK;

        // Test Clone
        let cloned = status;
        assert_eq!(status, cloned);

        // Test Copy (implicit through Clone)
        let copied = status;
        assert_eq!(status, copied);

        // Test PartialEq
        assert_eq!(fmi3Status::fmi3OK, fmi3Status::fmi3OK);
        assert_ne!(fmi3Status::fmi3OK, fmi3Status::fmi3Error);
    }

    // =============================================================================
    // Boundary and Stress Tests
    // =============================================================================

    /// Test conversion with values just outside the valid range
    #[test]
    fn test_boundary_values_around_valid_range() {
        // Just before valid range
        assert_eq!(fmi3Status::from(-1), fmi3Status::fmi3Error);

        // Start of valid range
        assert_eq!(fmi3Status::from(0), fmi3Status::fmi3OK);

        // End of valid range
        assert_eq!(fmi3Status::from(4), fmi3Status::fmi3Fatal);

        // Just after valid range
        assert_eq!(fmi3Status::from(5), fmi3Status::fmi3Error);
    }

    /// Test multiple conversions in sequence to ensure no state corruption
    #[test]
    fn test_sequential_conversions() {
        // Perform multiple conversions to ensure consistency
        for _ in 0..100 {
            assert_eq!(fmi3Status::from(0), fmi3Status::fmi3OK);
            assert_eq!(fmi3Status::from(ProtoStatus::Error), fmi3Status::fmi3Error);
            assert_eq!(
                ProtoStatus::from(fmi3Status::fmi3Warning),
                ProtoStatus::Warning
            );
        }
    }

    /// Test conversion with all values in a single test for coverage
    #[test]
    fn test_comprehensive_conversion_coverage() {
        // Test all valid i32 values
        let valid_i32_values = [0, 1, 2, 3, 4];
        let expected_fmi = [
            fmi3Status::fmi3OK,
            fmi3Status::fmi3Warning,
            fmi3Status::fmi3Discard,
            fmi3Status::fmi3Error,
            fmi3Status::fmi3Fatal,
        ];

        for (i32_val, expected) in valid_i32_values.iter().zip(expected_fmi.iter()) {
            assert_eq!(fmi3Status::from(*i32_val), *expected);
        }

        // Test all invalid i32 values default to Error
        let invalid_i32_values = [-1, 5, 10, 100, 1000];
        for invalid_val in invalid_i32_values {
            assert_eq!(fmi3Status::from(invalid_val), fmi3Status::fmi3Error);
        }
    }
}
