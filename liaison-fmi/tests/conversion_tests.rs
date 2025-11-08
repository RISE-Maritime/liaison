// Unit tests for conversion module
// These tests focus specifically on type conversions between proto and FMI types

use liaison_fmi::fmi3::fmi3Status;
use liaison_fmi::proto;

#[test]
fn test_all_proto_to_fmi_conversions() {
    // Test exhaustive conversion from proto::Status to fmi3Status
    let test_cases = vec![
        (proto::Status::Ok, fmi3Status::fmi3OK),
        (proto::Status::Warning, fmi3Status::fmi3Warning),
        (proto::Status::Discard, fmi3Status::fmi3Discard),
        (proto::Status::Error, fmi3Status::fmi3Error),
        (proto::Status::Fatal, fmi3Status::fmi3Fatal),
    ];

    for (proto_status, expected_fmi_status) in test_cases {
        let result: fmi3Status = proto_status.into();
        assert_eq!(
            result, expected_fmi_status,
            "Conversion from {:?} failed",
            proto_status
        );
    }
}

#[test]
fn test_all_fmi_to_proto_conversions() {
    // Test exhaustive conversion from fmi3Status to proto::Status
    let test_cases = vec![
        (fmi3Status::fmi3OK, proto::Status::Ok),
        (fmi3Status::fmi3Warning, proto::Status::Warning),
        (fmi3Status::fmi3Discard, proto::Status::Discard),
        (fmi3Status::fmi3Error, proto::Status::Error),
        (fmi3Status::fmi3Fatal, proto::Status::Fatal),
    ];

    for (fmi_status, expected_proto_status) in test_cases {
        let result: proto::Status = fmi_status.into();
        assert_eq!(
            result, expected_proto_status,
            "Conversion from {:?} failed",
            fmi_status
        );
    }
}

#[test]
fn test_i32_to_fmi_status_valid_values() {
    // Test conversion from valid i32 values to fmi3Status
    let test_cases = vec![
        (0, fmi3Status::fmi3OK),
        (1, fmi3Status::fmi3Warning),
        (2, fmi3Status::fmi3Discard),
        (3, fmi3Status::fmi3Error),
        (4, fmi3Status::fmi3Fatal),
    ];

    for (i32_val, expected_status) in test_cases {
        let result: fmi3Status = i32_val.into();
        assert_eq!(
            result, expected_status,
            "Conversion from i32 {} failed",
            i32_val
        );
    }
}

#[test]
fn test_i32_to_fmi_status_invalid_values() {
    // Test that invalid i32 values default to fmi3Error
    let invalid_values = vec![-1, 5, 10, 100, 999, i32::MIN, i32::MAX];

    for i32_val in invalid_values {
        let result: fmi3Status = i32_val.into();
        assert_eq!(
            result,
            fmi3Status::fmi3Error,
            "Invalid i32 {} should convert to fmi3Error",
            i32_val
        );
    }
}

#[test]
fn test_bidirectional_conversion_consistency() {
    // Test that converting proto -> fmi -> proto preserves the value
    let proto_statuses = vec![
        proto::Status::Ok,
        proto::Status::Warning,
        proto::Status::Discard,
        proto::Status::Error,
        proto::Status::Fatal,
    ];

    for original_proto in proto_statuses {
        let fmi: fmi3Status = original_proto.into();
        let back_to_proto: proto::Status = fmi.into();
        assert_eq!(
            back_to_proto, original_proto,
            "Round-trip conversion failed for {:?}",
            original_proto
        );
    }
}

#[test]
fn test_fmi_status_numeric_values() {
    // Verify the numeric representation of fmi3Status enum
    assert_eq!(fmi3Status::fmi3OK as i32, 0);
    assert_eq!(fmi3Status::fmi3Warning as i32, 1);
    assert_eq!(fmi3Status::fmi3Discard as i32, 2);
    assert_eq!(fmi3Status::fmi3Error as i32, 3);
    assert_eq!(fmi3Status::fmi3Fatal as i32, 4);
}

#[test]
fn test_proto_status_numeric_values() {
    // Verify the numeric representation of proto::Status enum
    assert_eq!(proto::Status::Ok as i32, 0);
    assert_eq!(proto::Status::Warning as i32, 1);
    assert_eq!(proto::Status::Discard as i32, 2);
    assert_eq!(proto::Status::Error as i32, 3);
    assert_eq!(proto::Status::Fatal as i32, 4);
}

#[test]
fn test_conversion_pattern_matching() {
    // Test that conversions work correctly in pattern matching contexts
    let proto_status = proto::Status::Ok;
    let fmi_status: fmi3Status = proto_status.into();

    match fmi_status {
        fmi3Status::fmi3OK => assert!(true),
        _ => panic!("Expected fmi3OK"),
    }
}
