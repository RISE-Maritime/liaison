// Comprehensive benchmarks for liaison-server
// Tests performance of instance manager, FMU creator operations, and server components

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use liaison_server::instance_manager::{InstanceManager, InstanceError};
use liaison_server::proto;
use prost::Message;
use std::f64::consts::PI;
use std::ffi::c_void;

// =============================================================================
// Instance Manager Benchmarks
// =============================================================================

/// Benchmark instance manager add operation
fn bench_instance_manager_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("instance_manager_add");

    group.bench_function("single_add", |b| {
        b.iter_batched(
            || {
                let manager = InstanceManager::new();
                let ptr = 0x1000 as *mut c_void;
                (manager, ptr)
            },
            |(manager, ptr)| {
                let index = manager.add_instance(black_box(ptr));
                black_box(index)
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Batch add operations
    for count in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(
            BenchmarkId::new("batch_add", count),
            count,
            |b, &count| {
                b.iter_batched(
                    || {
                        let manager = InstanceManager::new();
                        (manager, count)
                    },
                    |(manager, count)| {
                        for i in 0..count {
                            let ptr = (i * 0x1000) as *mut c_void;
                            manager.add_instance(black_box(ptr));
                        }
                        black_box(manager)
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark instance manager get operation
fn bench_instance_manager_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("instance_manager_get");

    // Single get - successful
    group.bench_function("single_get_success", |b| {
        let manager = InstanceManager::new();
        let ptr = 0x1000 as *mut c_void;
        let index = manager.add_instance(ptr);
        b.iter(|| {
            let result = manager.get_instance(black_box(index));
            black_box(result)
        });
    });

    // Single get - not found
    group.bench_function("single_get_not_found", |b| {
        let manager = InstanceManager::new();
        b.iter(|| {
            let result = manager.get_instance(black_box(999));
            black_box(result)
        });
    });

    // Single get - invalid index
    group.bench_function("single_get_invalid_index", |b| {
        let manager = InstanceManager::new();
        b.iter(|| {
            let result = manager.get_instance(black_box(-1));
            black_box(result)
        });
    });

    // Multiple gets from populated manager
    for instance_count in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("multiple_get_random", instance_count),
            instance_count,
            |b, &instance_count| {
                let manager = InstanceManager::new();
                let indices: Vec<i32> = (0..instance_count)
                    .map(|i| {
                        let ptr = (i * 0x1000) as *mut c_void;
                        manager.add_instance(ptr)
                    })
                    .collect();

                b.iter(|| {
                    // Get all instances
                    for &index in &indices {
                        let result = manager.get_instance(black_box(index));
                        let _ = black_box(result);
                    }
                });
            },
        );
    }

    // Sequential gets
    group.bench_function("sequential_gets_100", |b| {
        let manager = InstanceManager::new();
        for i in 0..100 {
            let ptr = (i * 0x1000) as *mut c_void;
            manager.add_instance(ptr);
        }

        b.iter(|| {
            for i in 0..100 {
                let result = manager.get_instance(black_box(i));
                let _ = black_box(result);
            }
        });
    });

    group.finish();
}

/// Benchmark instance manager remove operation
fn bench_instance_manager_remove(c: &mut Criterion) {
    let mut group = c.benchmark_group("instance_manager_remove");

    group.bench_function("single_remove_success", |b| {
        b.iter_batched(
            || {
                let manager = InstanceManager::new();
                let ptr = 0x1000 as *mut c_void;
                let index = manager.add_instance(ptr);
                (manager, index)
            },
            |(manager, index)| {
                let result = manager.remove_instance(black_box(index));
                black_box(result)
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("single_remove_not_found", |b| {
        b.iter_batched(
            InstanceManager::new,
            |manager| {
                let result = manager.remove_instance(black_box(999));
                black_box(result)
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Batch remove
    for count in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(
            BenchmarkId::new("batch_remove", count),
            count,
            |b, &count| {
                b.iter_batched(
                    || {
                        let manager = InstanceManager::new();
                        let indices: Vec<i32> = (0..count)
                            .map(|i| {
                                let ptr = (i * 0x1000) as *mut c_void;
                                manager.add_instance(ptr)
                            })
                            .collect();
                        (manager, indices)
                    },
                    |(manager, indices)| {
                        for index in indices {
                            manager.remove_instance(black_box(index)).ok();
                        }
                        black_box(manager)
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark instance manager query operations
fn bench_instance_manager_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("instance_manager_queries");

    // instance_count
    for count in [0, 10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("instance_count", count),
            count,
            |b, &count| {
                let manager = InstanceManager::new();
                for i in 0..count {
                    let ptr = (i * 0x1000) as *mut c_void;
                    manager.add_instance(ptr);
                }
                b.iter(|| {
                    let count = manager.instance_count();
                    black_box(count)
                });
            },
        );
    }

    // contains_instance - exists
    group.bench_function("contains_instance_exists", |b| {
        let manager = InstanceManager::new();
        let ptr = 0x1000 as *mut c_void;
        let index = manager.add_instance(ptr);
        b.iter(|| {
            let exists = manager.contains_instance(black_box(index));
            black_box(exists)
        });
    });

    // contains_instance - not exists
    group.bench_function("contains_instance_not_exists", |b| {
        let manager = InstanceManager::new();
        b.iter(|| {
            let exists = manager.contains_instance(black_box(999));
            black_box(exists)
        });
    });

    group.finish();
}

/// Benchmark instance manager mixed workload
fn bench_instance_manager_mixed_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("instance_manager_mixed");

    // Realistic simulation: add, get, update, remove pattern
    group.bench_function("simulation_pattern_small", |b| {
        b.iter_batched(
            InstanceManager::new,
            |manager| {
                // Add 10 instances
                let mut indices = Vec::new();
                for i in 0..10 {
                    let ptr = (i * 0x1000) as *mut c_void;
                    let index = manager.add_instance(ptr);
                    indices.push(index);
                }

                // Get each instance 100 times (simulating reads)
                for _ in 0..100 {
                    for &index in &indices {
                        manager.get_instance(index).ok();
                    }
                }

                // Remove half
                for &index in indices.iter().take(5) {
                    manager.remove_instance(index).ok();
                }

                black_box(manager)
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("simulation_pattern_large", |b| {
        b.iter_batched(
            InstanceManager::new,
            |manager| {
                // Add 100 instances
                let mut indices = Vec::new();
                for i in 0..100 {
                    let ptr = (i * 0x1000) as *mut c_void;
                    let index = manager.add_instance(ptr);
                    indices.push(index);
                }

                // Get each instance 10 times
                for _ in 0..10 {
                    for &index in &indices {
                        manager.get_instance(index).ok();
                    }
                }

                // Remove half
                for &index in indices.iter().take(50) {
                    manager.remove_instance(index).ok();
                }

                black_box(manager)
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

/// Benchmark instance manager thread safety overhead
fn bench_instance_manager_concurrent_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("instance_manager_concurrent");

    // Clone operation (cheap Arc clone)
    group.bench_function("clone_manager", |b| {
        let manager = InstanceManager::new();
        b.iter(|| {
            let cloned = manager.clone();
            black_box(cloned)
        });
    });

    // Multiple clones accessing same data
    group.bench_function("multi_clone_access", |b| {
        let manager = InstanceManager::new();
        for i in 0..10 {
            let ptr = (i * 0x1000) as *mut c_void;
            manager.add_instance(ptr);
        }

        b.iter(|| {
            let clone1 = manager.clone();
            let clone2 = manager.clone();
            let clone3 = manager.clone();

            // Simulate concurrent access
            let c1 = clone1.instance_count();
            let c2 = clone2.instance_count();
            let c3 = clone3.instance_count();

            black_box((c1, c2, c3))
        });
    });

    group.finish();
}

// =============================================================================
// Protobuf Message Benchmarks (Server-specific)
// =============================================================================

/// Benchmark server-side protobuf operations
fn bench_server_protobuf_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("server_protobuf");

    // Instance message encoding
    group.bench_function("instance_message_encode", |b| {
        let msg = proto::Fmi3InstanceMessage { instance_index: 42 };
        b.iter(|| {
            let mut buf = Vec::new();
            black_box(&msg).encode(&mut buf).unwrap();
            black_box(buf)
        });
    });

    // Instance message decoding
    group.bench_function("instance_message_decode", |b| {
        let msg = proto::Fmi3InstanceMessage { instance_index: 42 };
        let mut buf = Vec::new();
        msg.encode(&mut buf).unwrap();
        b.iter(|| {
            let decoded = proto::Fmi3InstanceMessage::decode(black_box(buf.as_slice())).unwrap();
            black_box(decoded)
        });
    });

    // Status message operations
    group.bench_function("status_message_roundtrip", |b| {
        let msg = proto::Fmi3StatusMessage {
            status: proto::Status::Ok as i32,
        };
        b.iter(|| {
            let mut buf = Vec::new();
            black_box(&msg).encode(&mut buf).unwrap();
            let decoded = proto::Fmi3StatusMessage::decode(buf.as_slice()).unwrap();
            black_box(decoded)
        });
    });

    // Large message handling - Float64
    for size in [1, 10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("float64_output_encode", size),
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
                    black_box(buf)
                });
            },
        );
    }

    group.finish();
}

// =============================================================================
// FMU Creator Benchmarks (JSON Operations)
// =============================================================================

/// Benchmark JSON configuration operations
fn bench_json_config_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_config");

    // Parse simple config
    group.bench_function("parse_simple_config", |b| {
        let config_str = r#"{
            "responderId": "test-responder-123",
            "name": "TestModel"
        }"#;
        b.iter(|| {
            let config: serde_json::Value =
                serde_json::from_str(black_box(config_str)).unwrap();
            black_box(config)
        });
    });

    // Parse complex config with Zenoh settings
    group.bench_function("parse_complex_config", |b| {
        let config_str = r#"{
            "responderId": "test-responder-456",
            "name": "ComplexModel",
            "zenohConfig": {
                "mode": "client",
                "connect": {
                    "endpoints": ["tcp/localhost:7447"]
                },
                "transport": {
                    "link": {
                        "tls": {
                            "connect_certificate": "client.pem",
                            "connect_private_key": "client.key",
                            "root_ca_certificate": "ca.pem"
                        }
                    }
                }
            }
        }"#;
        b.iter(|| {
            let config: serde_json::Value =
                serde_json::from_str(black_box(config_str)).unwrap();
            black_box(config)
        });
    });

    // Serialize config
    group.bench_function("serialize_config", |b| {
        let config = serde_json::json!({
            "responderId": "test-responder",
            "name": "TestModel",
            "zenohConfig": {
                "mode": "client",
                "connect": {
                    "endpoints": ["tcp/localhost:7447"]
                }
            }
        });
        b.iter(|| {
            let serialized = serde_json::to_string(black_box(&config)).unwrap();
            black_box(serialized)
        });
    });

    // Serialize pretty
    group.bench_function("serialize_config_pretty", |b| {
        let config = serde_json::json!({
            "responderId": "test-responder",
            "name": "TestModel",
            "zenohConfig": {
                "mode": "client"
            }
        });
        b.iter(|| {
            let serialized = serde_json::to_string_pretty(black_box(&config)).unwrap();
            black_box(serialized)
        });
    });

    // Extract field from config
    group.bench_function("extract_responder_id", |b| {
        let config = serde_json::json!({
            "responderId": "test-responder-789",
            "name": "TestModel"
        });
        b.iter(|| {
            let responder_id = black_box(&config)["responderId"].as_str();
            black_box(responder_id)
        });
    });

    // Modify config (add metadata)
    group.bench_function("modify_config_add_metadata", |b| {
        b.iter_batched(
            || {
                serde_json::json!({
                    "mode": "client",
                    "connect": {
                        "endpoints": ["tcp/localhost:7447"]
                    }
                })
            },
            |mut config| {
                if config.get("metadata").is_none() {
                    config["metadata"] = serde_json::json!({});
                }
                config["metadata"]["name"] = serde_json::json!("TestModel");
                black_box(config)
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

/// Benchmark path extraction and manipulation
fn bench_path_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("path_operations");

    // Extract model name from path
    group.bench_function("extract_model_name", |b| {
        let path = std::path::PathBuf::from("/path/to/MyModel.fmu");
        b.iter(|| {
            let model_name = black_box(&path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap();
            black_box(model_name)
        });
    });

    // Extract filename from certificate path
    group.bench_function("extract_cert_filename", |b| {
        let path = std::path::PathBuf::from("/etc/certs/client-certificate.pem");
        b.iter(|| {
            let filename = black_box(&path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap();
            black_box(filename)
        });
    });

    // Build output FMU path
    group.bench_function("build_output_path", |b| {
        let model_name = "TestModel";
        b.iter(|| {
            let output_path =
                std::path::PathBuf::from(format!("{}Liaison.fmu", black_box(model_name)));
            black_box(output_path)
        });
    });

    // Format library paths
    group.bench_function("format_library_paths", |b| {
        let model_name = "TestModel";
        b.iter(|| {
            let linux_lib =
                format!("binaries/x86_64-linux/{}.so", black_box(model_name));
            let windows_lib = format!(
                "binaries/x86_64-windows/{}.dll",
                black_box(model_name)
            );
            black_box((linux_lib, windows_lib))
        });
    });

    group.finish();
}

// =============================================================================
// Error Handling Benchmarks
// =============================================================================

/// Benchmark error creation and handling
fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");

    // Create InstanceError variants
    group.bench_function("create_instance_not_found_error", |b| {
        b.iter(|| {
            let error = InstanceError::InstanceNotFound(black_box(42));
            black_box(error)
        });
    });

    group.bench_function("create_invalid_index_error", |b| {
        b.iter(|| {
            let error = InstanceError::InvalidIndex(black_box(-1));
            black_box(error)
        });
    });

    // Format error messages
    group.bench_function("format_error_message", |b| {
        let error = InstanceError::InstanceNotFound(42);
        b.iter(|| {
            let message = format!("{}", black_box(&error));
            black_box(message)
        });
    });

    // Result pattern matching
    group.bench_function("result_match_ok", |b| {
        let result: Result<i32, InstanceError> = Ok(42);
        b.iter(|| match black_box(&result) {
            Ok(val) => black_box(*val),
            Err(_) => 0,
        });
    });

    group.bench_function("result_match_err", |b| {
        let result: Result<i32, InstanceError> = Err(InstanceError::InstanceNotFound(999));
        b.iter(|| match black_box(&result) {
            Ok(val) => black_box(*val),
            Err(_) => 0,
        });
    });

    group.finish();
}

// =============================================================================
// String and Vector Operations Benchmarks
// =============================================================================

/// Benchmark common string operations
fn bench_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_operations");

    // String formatting
    group.bench_function("format_key_expression", |b| {
        let responder_id = "test-responder-123";
        let function_name = "fmi3DoStep";
        b.iter(|| {
            let expr = format!(
                "rpc/{}/{}",
                black_box(responder_id),
                black_box(function_name)
            );
            black_box(expr)
        });
    });

    // String cloning
    for len in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("string_clone", len), len, |b, &len| {
            let s = "X".repeat(len);
            b.iter(|| {
                let cloned = black_box(&s).clone();
                black_box(cloned)
            });
        });
    }

    // String to_string conversion
    group.bench_function("str_to_string", |b| {
        let s = "test-string-conversion";
        b.iter(|| {
            let owned = black_box(s).to_string();
            black_box(owned)
        });
    });

    group.finish();
}

/// Benchmark vector operations
fn bench_vector_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_operations");

    // Vector creation with capacity
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("vec_with_capacity", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let v: Vec<i32> = Vec::with_capacity(black_box(size));
                    black_box(v)
                });
            },
        );
    }

    // Vector population
    for size in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::new("vec_populate", size), size, |b, &size| {
            b.iter(|| {
                let v: Vec<i32> = (0..black_box(size)).collect();
                black_box(v)
            });
        });
    }

    // Vector of floats
    for size in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("vec_floats", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let v = vec![PI; black_box(size) as usize];
                    black_box(v)
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
    instance_manager_benches,
    bench_instance_manager_add,
    bench_instance_manager_get,
    bench_instance_manager_remove,
    bench_instance_manager_queries,
    bench_instance_manager_mixed_workload,
    bench_instance_manager_concurrent_access,
);

criterion_group!(
    server_protobuf_benches,
    bench_server_protobuf_operations,
);

criterion_group!(
    fmu_creator_benches,
    bench_json_config_operations,
    bench_path_operations,
);

criterion_group!(
    utility_benches,
    bench_error_handling,
    bench_string_operations,
    bench_vector_operations,
);

criterion_main!(
    instance_manager_benches,
    server_protobuf_benches,
    fmu_creator_benches,
    utility_benches,
);
