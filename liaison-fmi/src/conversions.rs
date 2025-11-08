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

    #[test]
    fn test_proto_to_fmi_status() {
        assert_eq!(fmi3Status::from(ProtoStatus::Ok), fmi3Status::fmi3OK);
        assert_eq!(fmi3Status::from(ProtoStatus::Warning), fmi3Status::fmi3Warning);
        assert_eq!(fmi3Status::from(ProtoStatus::Discard), fmi3Status::fmi3Discard);
        assert_eq!(fmi3Status::from(ProtoStatus::Error), fmi3Status::fmi3Error);
        assert_eq!(fmi3Status::from(ProtoStatus::Fatal), fmi3Status::fmi3Fatal);
    }

    #[test]
    fn test_fmi_to_proto_status() {
        assert_eq!(ProtoStatus::from(fmi3Status::fmi3OK), ProtoStatus::Ok);
        assert_eq!(ProtoStatus::from(fmi3Status::fmi3Warning), ProtoStatus::Warning);
        assert_eq!(ProtoStatus::from(fmi3Status::fmi3Discard), ProtoStatus::Discard);
        assert_eq!(ProtoStatus::from(fmi3Status::fmi3Error), ProtoStatus::Error);
        assert_eq!(ProtoStatus::from(fmi3Status::fmi3Fatal), ProtoStatus::Fatal);
    }

    #[test]
    fn test_i32_to_fmi_status() {
        assert_eq!(fmi3Status::from(0), fmi3Status::fmi3OK);
        assert_eq!(fmi3Status::from(1), fmi3Status::fmi3Warning);
        assert_eq!(fmi3Status::from(2), fmi3Status::fmi3Discard);
        assert_eq!(fmi3Status::from(3), fmi3Status::fmi3Error);
        assert_eq!(fmi3Status::from(4), fmi3Status::fmi3Fatal);
        assert_eq!(fmi3Status::from(99), fmi3Status::fmi3Error); // Unknown value defaults to Error
    }
}
