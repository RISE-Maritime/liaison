// Comprehensive benchmarks for liaison-fmi client library
// Tests performance of protobuf operations, type conversions, and core functionality

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
// Conversions are accessed via fmi3 module
use liaisonfmu::fmi3::fmi3Status;
use liaisonfmu::proto;
use prost::Message;
use std::f64::consts::PI;

// =============================================================================
// Protobuf Serialization/Deserialization Benchmarks
// =============================================================================

/// Benchmark protobuf message encoding (serialization)
fn bench_protobuf_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("protobuf_encode");

    // Simple message - fmi3InstanceMessage
    group.bench_function("instance_message", |b| {
        let msg = proto::Fmi3InstanceMessage { instance_index: 42 };
        b.iter(|| {
            let mut buf = Vec::new();
            black_box(&msg).encode(&mut buf).unwrap();
            black_box(buf)
        });
    });

    // Status message
    group.bench_function("status_message", |b| {
        let msg = proto::Fmi3StatusMessage {
            status: proto::Status::Ok as i32,
        };
        b.iter(|| {
            let mut buf = Vec::new();
            black_box(&msg).encode(&mut buf).unwrap();
            black_box(buf)
        });
    });

    // Complex message - DoStep with all fields
    group.bench_function("do_step_message", |b| {
        let msg = proto::Fmi3DoStepMessage {
            instance_index: 42,
            current_communication_point: 0.0,
            communication_step_size: 0.01,
            no_set_fmu_state_prior_to_current_point: true,
            event_handling_needed: false,
            terminate_simulation: false,
            early_return: false,
            last_successful_time: 0.0,
        };
        b.iter(|| {
            let mut buf = Vec::new();
            black_box(&msg).encode(&mut buf).unwrap();
            black_box(buf)
        });
    });

    // Float64 message with varying sizes
    for size in [1, 10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("float64_set_message", size),
            size,
            |b, &size| {
                let msg = proto::Fmi3SetFloat64InputMessage {
                    instance_index: 42,
                    value_references: (0..size).collect(),
                    n_value_references: size,
                    values: vec![PI; size as usize],
                    n_values: size,
                };
                b.iter(|| {
                    let mut buf = Vec::new();
                    black_box(&msg).encode(&mut buf).unwrap();
                    black_box(buf)
                });
            },
        );
    }

    // Int32 message with varying sizes
    for size in [1, 10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("int32_set_message", size),
            size,
            |b, &size| {
                let msg = proto::Fmi3SetInt32InputMessage {
                    instance_index: 42,
                    value_references: (0..size).collect(),
                    n_value_references: size,
                    values: vec![42; size as usize],
                    n_values: size,
                };
                b.iter(|| {
                    let mut buf = Vec::new();
                    black_box(&msg).encode(&mut buf).unwrap();
                    black_box(buf)
                });
            },
        );
    }

    // Instantiation message with strings
    group.bench_function("instantiate_cosimulation", |b| {
        let msg = proto::Fmi3InstantiateCoSimulationMessage {
            instance_name: "TestInstance".to_string(),
            instantiation_token: "{12345678-1234-1234-1234-123456789012}".to_string(),
            resource_path: "/path/to/resources".to_string(),
            visible: false,
            logging_on: true,
            event_mode_used: false,
            early_return_allowed: true,
            required_intermediate_variables: vec![],
            n_required_intermediate_variables: 0,
        };
        b.iter(|| {
            let mut buf = Vec::new();
            black_box(&msg).encode(&mut buf).unwrap();
            black_box(buf)
        });
    });

    // Log message
    group.bench_function("log_message", |b| {
        let msg = proto::LogMessage {
            status: proto::Status::Warning as i32,
            category: "TestCategory".to_string(),
            message: "This is a test log message with some content".to_string(),
        };
        b.iter(|| {
            let mut buf = Vec::new();
            black_box(&msg).encode(&mut buf).unwrap();
            black_box(buf)
        });
    });

    group.finish();
}

/// Benchmark protobuf message decoding (deserialization)
fn bench_protobuf_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("protobuf_decode");

    // Simple message - fmi3InstanceMessage
    group.bench_function("instance_message", |b| {
        let msg = proto::Fmi3InstanceMessage { instance_index: 42 };
        let mut buf = Vec::new();
        msg.encode(&mut buf).unwrap();
        b.iter(|| {
            let decoded =
                proto::Fmi3InstanceMessage::decode(black_box(buf.as_slice())).unwrap();
            black_box(decoded)
        });
    });

    // Status message
    group.bench_function("status_message", |b| {
        let msg = proto::Fmi3StatusMessage {
            status: proto::Status::Ok as i32,
        };
        let mut buf = Vec::new();
        msg.encode(&mut buf).unwrap();
        b.iter(|| {
            let decoded = proto::Fmi3StatusMessage::decode(black_box(buf.as_slice())).unwrap();
            black_box(decoded)
        });
    });

    // Complex message - DoStep
    group.bench_function("do_step_message", |b| {
        let msg = proto::Fmi3DoStepMessage {
            instance_index: 42,
            current_communication_point: 0.0,
            communication_step_size: 0.01,
            no_set_fmu_state_prior_to_current_point: true,
            event_handling_needed: false,
            terminate_simulation: false,
            early_return: false,
            last_successful_time: 0.0,
        };
        let mut buf = Vec::new();
        msg.encode(&mut buf).unwrap();
        b.iter(|| {
            let decoded = proto::Fmi3DoStepMessage::decode(black_box(buf.as_slice())).unwrap();
            black_box(decoded)
        });
    });

    // Float64 message with varying sizes
    for size in [1, 10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("float64_get_message", size),
            size,
            |b, &size| {
                let msg = proto::Fmi3GetFloat64OutputMessage {
                    values: vec![PI; size as usize],
                    n_values: size,
                    status: proto::Status::Ok as i32,
                };
                let mut buf = Vec::new();
                msg.encode(&mut buf).unwrap();
                b.iter(|| {
                    let decoded =
                        proto::Fmi3GetFloat64OutputMessage::decode(black_box(buf.as_slice()))
                            .unwrap();
                    black_box(decoded)
                });
            },
        );
    }

    // Log message
    group.bench_function("log_message", |b| {
        let msg = proto::LogMessage {
            status: proto::Status::Warning as i32,
            category: "TestCategory".to_string(),
            message: "This is a test log message with some content".to_string(),
        };
        let mut buf = Vec::new();
        msg.encode(&mut buf).unwrap();
        b.iter(|| {
            let decoded = proto::LogMessage::decode(black_box(buf.as_slice())).unwrap();
            black_box(decoded)
        });
    });

    group.finish();
}

/// Benchmark round-trip encoding and decoding
fn bench_protobuf_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("protobuf_roundtrip");

    group.bench_function("instance_message", |b| {
        let msg = proto::Fmi3InstanceMessage { instance_index: 42 };
        b.iter(|| {
            let mut buf = Vec::new();
            black_box(&msg).encode(&mut buf).unwrap();
            let decoded = proto::Fmi3InstanceMessage::decode(buf.as_slice()).unwrap();
            black_box(decoded)
        });
    });

    group.bench_function("do_step_message", |b| {
        let msg = proto::Fmi3DoStepMessage {
            instance_index: 42,
            current_communication_point: 0.0,
            communication_step_size: 0.01,
            no_set_fmu_state_prior_to_current_point: true,
            event_handling_needed: false,
            terminate_simulation: false,
            early_return: false,
            last_successful_time: 0.0,
        };
        b.iter(|| {
            let mut buf = Vec::new();
            black_box(&msg).encode(&mut buf).unwrap();
            let decoded = proto::Fmi3DoStepMessage::decode(buf.as_slice()).unwrap();
            black_box(decoded)
        });
    });

    // Float64 roundtrip with varying sizes
    for size in [1, 10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("float64_message", size),
            size,
            |b, &size| {
                let msg = proto::Fmi3GetFloat64OutputMessage {
                    values: vec![PI; size as usize],
                    n_values: size,
                    status: proto::Status::Ok as i32,
                };
                b.iter(|| {
                    let mut buf = Vec::new();
                    black_box(&msg).encode(&mut buf).unwrap();
                    let decoded =
                        proto::Fmi3GetFloat64OutputMessage::decode(buf.as_slice()).unwrap();
                    black_box(decoded)
                });
            },
        );
    }

    group.finish();
}

// =============================================================================
// Type Conversion Benchmarks
// =============================================================================

/// Benchmark status type conversions
fn bench_status_conversions(c: &mut Criterion) {
    let mut group = c.benchmark_group("status_conversions");

    // ProtoStatus -> fmi3Status
    group.bench_function("proto_to_fmi_ok", |b| {
        b.iter(|| {
            let status: fmi3Status = black_box(proto::Status::Ok).into();
            black_box(status)
        });
    });

    group.bench_function("proto_to_fmi_error", |b| {
        b.iter(|| {
            let status: fmi3Status = black_box(proto::Status::Error).into();
            black_box(status)
        });
    });

    group.bench_function("proto_to_fmi_fatal", |b| {
        b.iter(|| {
            let status: fmi3Status = black_box(proto::Status::Fatal).into();
            black_box(status)
        });
    });

    // fmi3Status -> ProtoStatus
    group.bench_function("fmi_to_proto_ok", |b| {
        b.iter(|| {
            let status: proto::Status = black_box(fmi3Status::fmi3OK).into();
            black_box(status)
        });
    });

    group.bench_function("fmi_to_proto_error", |b| {
        b.iter(|| {
            let status: proto::Status = black_box(fmi3Status::fmi3Error).into();
            black_box(status)
        });
    });

    // i32 -> fmi3Status
    group.bench_function("i32_to_fmi_valid", |b| {
        b.iter(|| {
            let status: fmi3Status = black_box(0_i32).into();
            black_box(status)
        });
    });

    group.bench_function("i32_to_fmi_invalid", |b| {
        b.iter(|| {
            let status: fmi3Status = black_box(999_i32).into();
            black_box(status)
        });
    });

    // Round-trip conversions
    group.bench_function("roundtrip_proto_fmi_proto", |b| {
        b.iter(|| {
            let proto_status = black_box(proto::Status::Warning);
            let fmi_status: fmi3Status = proto_status.into();
            let back_to_proto: proto::Status = fmi_status.into();
            black_box(back_to_proto)
        });
    });

    group.bench_function("roundtrip_fmi_proto_fmi", |b| {
        b.iter(|| {
            let fmi_status = black_box(fmi3Status::fmi3Discard);
            let proto_status: proto::Status = fmi_status.into();
            let back_to_fmi: fmi3Status = proto_status.into();
            black_box(back_to_fmi)
        });
    });

    // Batch conversions
    group.throughput(Throughput::Elements(5));
    group.bench_function("batch_proto_to_fmi", |b| {
        let statuses = vec![
            proto::Status::Ok,
            proto::Status::Warning,
            proto::Status::Discard,
            proto::Status::Error,
            proto::Status::Fatal,
        ];
        b.iter(|| {
            let converted: Vec<fmi3Status> = black_box(&statuses)
                .iter()
                .map(|&s| s.into())
                .collect();
            black_box(converted)
        });
    });

    group.finish();
}

// =============================================================================
// Message Creation and Manipulation Benchmarks
// =============================================================================

/// Benchmark creating and populating protobuf messages
fn bench_message_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_creation");

    group.bench_function("simple_instance_message", |b| {
        b.iter(|| {
            let msg = proto::Fmi3InstanceMessage {
                instance_index: black_box(42),
            };
            black_box(msg)
        });
    });

    group.bench_function("do_step_message", |b| {
        b.iter(|| {
            let msg = proto::Fmi3DoStepMessage {
                instance_index: black_box(42),
                current_communication_point: black_box(1.0),
                communication_step_size: black_box(0.01),
                no_set_fmu_state_prior_to_current_point: black_box(true),
                event_handling_needed: black_box(false),
                terminate_simulation: black_box(false),
                early_return: black_box(false),
                last_successful_time: black_box(0.0),
            };
            black_box(msg)
        });
    });

    // Vector-based messages with varying sizes
    for size in [1, 10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("float64_message_with_vectors", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let msg = proto::Fmi3SetFloat64InputMessage {
                        instance_index: black_box(42),
                        value_references: black_box((0..size).collect()),
                        n_value_references: black_box(size),
                        values: black_box(vec![PI; size as usize]),
                        n_values: black_box(size),
                    };
                    black_box(msg)
                });
            },
        );
    }

    group.bench_function("instantiate_message_with_strings", |b| {
        b.iter(|| {
            let msg = proto::Fmi3InstantiateCoSimulationMessage {
                instance_name: black_box("TestInstance".to_string()),
                instantiation_token: black_box(
                    "{12345678-1234-1234-1234-123456789012}".to_string(),
                ),
                resource_path: black_box("/path/to/resources".to_string()),
                visible: black_box(false),
                logging_on: black_box(true),
                event_mode_used: black_box(false),
                early_return_allowed: black_box(true),
                required_intermediate_variables: black_box(vec![]),
                n_required_intermediate_variables: black_box(0),
            };
            black_box(msg)
        });
    });

    group.finish();
}

/// Benchmark encoded message sizes
fn bench_encoded_len(c: &mut Criterion) {
    let mut group = c.benchmark_group("encoded_len");

    group.bench_function("instance_message", |b| {
        let msg = proto::Fmi3InstanceMessage { instance_index: 42 };
        b.iter(|| {
            let len = black_box(&msg).encoded_len();
            black_box(len)
        });
    });

    group.bench_function("do_step_message", |b| {
        let msg = proto::Fmi3DoStepMessage {
            instance_index: 42,
            current_communication_point: 0.0,
            communication_step_size: 0.01,
            no_set_fmu_state_prior_to_current_point: true,
            event_handling_needed: false,
            terminate_simulation: false,
            early_return: false,
            last_successful_time: 0.0,
        };
        b.iter(|| {
            let len = black_box(&msg).encoded_len();
            black_box(len)
        });
    });

    for size in [1, 10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("float64_message", size),
            size,
            |b, &size| {
                let msg = proto::Fmi3SetFloat64InputMessage {
                    instance_index: 42,
                    value_references: (0..size).collect(),
                    n_value_references: size,
                    values: vec![PI; size as usize],
                    n_values: size,
                };
                b.iter(|| {
                    let len = black_box(&msg).encoded_len();
                    black_box(len)
                });
            },
        );
    }

    group.finish();
}

// =============================================================================
// String Handling Benchmarks
// =============================================================================

/// Benchmark string operations in messages
fn bench_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_operations");

    // Creating messages with different string sizes
    for str_len in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("log_message_creation", str_len),
            str_len,
            |b, &str_len| {
                let message_text = "X".repeat(str_len);
                b.iter(|| {
                    let msg = proto::LogMessage {
                        status: black_box(proto::Status::Ok as i32),
                        category: black_box("Category".to_string()),
                        message: black_box(message_text.clone()),
                    };
                    black_box(msg)
                });
            },
        );
    }

    // Encoding messages with different string sizes
    for str_len in [10, 100, 1000].iter() {
        group.throughput(Throughput::Bytes(*str_len as u64));
        group.bench_with_input(
            BenchmarkId::new("log_message_encode", str_len),
            str_len,
            |b, &str_len| {
                let message_text = "X".repeat(str_len);
                let msg = proto::LogMessage {
                    status: proto::Status::Ok as i32,
                    category: "Category".to_string(),
                    message: message_text,
                };
                b.iter(|| {
                    let mut buf = Vec::new();
                    black_box(&msg).encode(&mut buf).unwrap();
                    black_box(buf)
                });
            },
        );
    }

    group.finish();
}

// =============================================================================
// Criterion Configuration
// =============================================================================

criterion_group!(
    protobuf_benches,
    bench_protobuf_encode,
    bench_protobuf_decode,
    bench_protobuf_roundtrip,
    bench_encoded_len,
);

criterion_group!(
    conversion_benches,
    bench_status_conversions,
);

criterion_group!(
    message_benches,
    bench_message_creation,
    bench_string_operations,
);

criterion_main!(
    protobuf_benches,
    conversion_benches,
    message_benches,
);
