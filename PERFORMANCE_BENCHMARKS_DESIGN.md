# Liaison FMI Performance Benchmarks Design

## Table of Contents
1. [Overview](#overview)
2. [Benchmark Categories](#benchmark-categories)
3. [Test Scenarios](#test-scenarios)
4. [Metrics and Success Criteria](#metrics-and-success-criteria)
5. [Implementation Structure](#implementation-structure)
6. [Running Benchmarks](#running-benchmarks)
7. [Results Analysis](#results-analysis)
8. [Fair Comparison Guidelines](#fair-comparison-guidelines)

---

## Overview

### Purpose

This document defines a comprehensive performance benchmarking framework for comparing the Rust and C++ implementations of Liaison FMI. The goal is to provide objective, reproducible performance measurements across multiple dimensions:

- **Memory Usage**: Heap allocations, memory footprint, leak detection
- **CPU Performance**: Function call overhead, serialization, network operations
- **Network Latency**: Round-trip time for FMI operations over Zenoh
- **Throughput**: Operations per second for high-frequency calls
- **Scalability**: Performance with multiple concurrent FMU instances

### Scope

**In Scope:**
- Microbenchmarks for individual components (serialization, FFI, etc.)
- End-to-end benchmarks for complete FMI workflows
- Memory profiling and allocation tracking
- Network latency and throughput measurements
- Multi-instance concurrency tests

**Out of Scope:**
- FMU simulation accuracy (functional correctness is verified by tests)
- Operating system-specific optimizations (we measure on Linux primarily)
- Network infrastructure performance (assumed constant)

### Goals

1. **Quantify Performance**: Provide concrete numbers for Rust vs C++ performance
2. **Identify Bottlenecks**: Find performance-critical paths in both implementations
3. **Guide Optimization**: Inform decisions about where to focus optimization efforts
4. **Validate Migration**: Ensure Rust implementation meets performance requirements
5. **Regression Detection**: Establish baselines for continuous performance monitoring

### Non-Goals

- This is not a competition between languages, but a practical assessment
- We are not trying to prove one implementation is "better" than the other
- The focus is on understanding performance characteristics, not marketing claims

---

## Benchmark Categories

### 1. Memory Usage Benchmarks

#### 1.1 Heap Allocation Tracking

**What to Measure:**
- Total heap allocations during FMU lifecycle
- Peak memory usage (RSS - Resident Set Size)
- Memory fragmentation over time
- Memory leak detection

**Test Scenarios:**
- Single FMU instance creation and destruction
- 10, 50, 100 concurrent FMU instances
- Long-running server (1 hour, 24 hours)
- Stress test: create/destroy 1000 instances

**Tools:**
- **Rust**: `jemalloc` with profiling, `valgrind --tool=massif`
- **C++**: `valgrind --tool=massif`, built-in allocator stats
- **Both**: `/usr/bin/time -v` for peak RSS, `heaptrack`

**Success Criteria:**
- Rust and C++ should be within 20% of each other for peak memory
- No memory leaks detected over 1 hour run
- Memory usage should be stable (not growing) over time

#### 1.2 Memory Footprint by Component

**What to Measure:**
- Zenoh session memory overhead
- Protobuf message buffer sizes
- Instance manager memory per instance
- Library loading memory cost

**Test Scenarios:**
- Isolated component benchmarks
- Incremental addition of components
- Before/after snapshots at each stage

**Tools:**
- `heaptrack`, `massif-visualizer`
- Custom memory tracking in Rust using allocator APIs

**Success Criteria:**
- Identify which components use the most memory
- Rust should have comparable or lower memory usage due to RAII

### 2. CPU Performance Benchmarks

#### 2.1 Microbenchmarks

**Protobuf Serialization/Deserialization:**
```
Scenarios:
- Empty message
- Small message (getVersion)
- Medium message (getFloat64 with 10 values)
- Large message (getFloat64 with 1000 values)
- Binary data (getBinary with 1KB, 10KB, 100KB, 1MB)

Metrics:
- Time per serialize operation (nanoseconds)
- Time per deserialize operation (nanoseconds)
- Throughput (messages/second)
- Memory allocated per operation
```

**FFI Boundary Overhead:**
```
Scenarios:
- C → Rust call (fmi3GetVersion)
- Rust → C call (callback invocation)
- Pointer casting and validation
- String conversion (CStr ↔ String)

Metrics:
- Time per FFI call (nanoseconds)
- Overhead vs direct function call
- Cache effects (hot vs cold)
```

**Zenoh Operation Performance:**
```
Scenarios:
- Publish operation (different payload sizes)
- Query/reply round-trip (local network)
- Subscriber callback invocation
- Session creation/destruction

Metrics:
- Time per operation (microseconds)
- Throughput (operations/second)
- Latency distribution (p50, p90, p99, p99.9)
```

**Type Conversions:**
```
Scenarios:
- FMI status enum ↔ Proto status
- Value arrays (Float64, Int32, Boolean)
- String allocations and copying
- Binary data copying

Metrics:
- Time per conversion (nanoseconds)
- Memory allocated per conversion
```

**Dynamic Library Loading:**
```
Scenarios:
- Load FMU library (cold start)
- Symbol resolution (33 FMI functions)
- Library unloading

Metrics:
- Total load time (milliseconds)
- Time per symbol resolution (microseconds)
- Memory overhead
```

#### 2.2 Macrobenchmarks

**Complete FMI Workflows:**
```
Scenarios:
1. Instantiation Workflow:
   - fmi3InstantiateCoSimulation
   - fmi3EnterInitializationMode
   - fmi3ExitInitializationMode

2. Simulation Step Workflow:
   - fmi3SetFloat64 (10 inputs)
   - fmi3DoStep
   - fmi3GetFloat64 (10 outputs)

3. Termination Workflow:
   - fmi3Terminate
   - fmi3FreeInstance

Metrics:
- Total workflow time (milliseconds)
- Time per function call
- Cumulative overhead vs direct FMU call
```

**Tools:**
- **Rust**: `criterion` crate (statistical benchmarking)
- **C++**: Google Benchmark or Catch2 benchmarks
- **Both**: `perf stat` for hardware counters

**Success Criteria:**
- Rust should be within 10% of C++ for CPU-bound operations
- Serialization should be comparable (both use similar protobuf codegen)
- FFI overhead should be minimal (<100ns per call)

### 3. Network Latency Benchmarks

#### 3.1 Round-Trip Latency

**What to Measure:**
- Client → Server → Client time for each FMI function
- Latency distribution over many calls
- Effect of message size on latency

**Test Scenarios:**
```
Network Configurations:
- Localhost (loopback)
- Same LAN (1 Gbps Ethernet)
- Cross-subnet (via router)
- With Zenoh router (client mode)

Message Sizes:
- Small: getVersion (~50 bytes)
- Medium: getFloat64 with 10 values (~200 bytes)
- Large: getFloat64 with 1000 values (~8 KB)
- XLarge: getBinary with 1 MB

Call Frequencies:
- Single call (cold)
- 100 sequential calls (warm)
- 10,000 calls (sustained)
```

**Metrics:**
- Minimum latency (best case)
- Median latency (p50)
- 90th percentile (p90)
- 99th percentile (p99)
- 99.9th percentile (p99.9)
- Maximum latency (worst case)
- Standard deviation

**Tools:**
- Custom latency measurement in benchmark code
- `tcpdump` or `wireshark` for packet-level analysis
- Zenoh built-in metrics (if available)

**Success Criteria:**
- Latency should be dominated by network, not implementation
- Rust and C++ should have similar latency distributions
- Target: <1ms median latency for localhost, small messages
- Target: <5ms median latency for LAN, small messages

#### 3.2 Latency Under Load

**What to Measure:**
- Latency degradation with multiple concurrent clients
- Server-side queuing delays
- Zenoh session congestion effects

**Test Scenarios:**
```
Concurrent Clients:
- 1, 5, 10, 20, 50 clients
- Each making requests at 10 Hz

Server Instances:
- 1, 10, 100 FMU instances loaded

Load Patterns:
- Steady state (constant rate)
- Burst (100 requests in 1 second, then idle)
- Ramp-up (1→50 clients over 1 minute)
```

**Metrics:**
- Latency at each concurrency level
- Server CPU utilization
- Server memory usage
- Network bandwidth utilization

**Success Criteria:**
- Latency should scale gracefully (sub-linear increase)
- No timeouts or dropped requests
- Server should handle at least 50 concurrent clients

### 4. Throughput Benchmarks

#### 4.1 Operations per Second

**What to Measure:**
- Maximum sustained request rate
- Aggregate throughput across multiple clients
- Throughput vs latency trade-off

**Test Scenarios:**
```
Single Client Throughput:
- Sequential requests (synchronous)
- Pipelined requests (async, if supported)
- Different operation types

Multi-Client Throughput:
- 10 clients, each measuring individual throughput
- Aggregate throughput calculation
- Fairness between clients

Operation Mix:
- 100% reads (getFloat64)
- 100% writes (setFloat64)
- 50/50 read/write mix
- Realistic simulation workload
```

**Metrics:**
- Requests per second (total)
- Requests per second per client
- Data throughput (MB/s)
- CPU efficiency (requests per CPU second)

**Tools:**
- Custom throughput benchmark
- `perf stat` for CPU time measurements
- Load testing tools (if applicable)

**Success Criteria:**
- Target: >1000 requests/second for small operations (localhost)
- Target: >100 MB/s for large data transfers
- Rust and C++ within 15% of each other

#### 4.2 Scalability with Instance Count

**What to Measure:**
- Throughput degradation as instance count increases
- Memory usage vs throughput trade-off
- Server resource saturation point

**Test Scenarios:**
```
Instance Counts:
- 1, 10, 50, 100, 200 FMU instances

Request Pattern:
- Round-robin across instances
- Random instance selection
- Targeted (single instance under load)

Metrics per Instance Count:
- Aggregate throughput
- Per-instance throughput
- Memory usage
- CPU usage
```

**Success Criteria:**
- Throughput should scale linearly up to 50 instances
- Resource usage should be predictable
- No instance starvation

### 5. Startup and Shutdown Performance

#### 5.1 Startup Latency

**What to Measure:**
- Time from process start to ready for requests
- FMU loading time
- Zenoh session initialization time
- First request latency (cold start)

**Test Scenarios:**
```
FMU Sizes:
- Small FMU (~1 MB, e.g., BouncingBall)
- Medium FMU (~10 MB)
- Large FMU (~100 MB)

Startup Phases:
1. Process launch
2. FMU extraction
3. Library loading
4. Zenoh session creation
5. Queryables declaration
6. First request handling
```

**Metrics:**
- Total startup time (seconds)
- Time per phase (milliseconds)
- Memory allocated during startup

**Tools:**
- Custom timing instrumentation
- `time` command
- `strace -T` for syscall timing

**Success Criteria:**
- Target: <5 seconds for small FMU (localhost)
- Target: <30 seconds for large FMU
- Rust and C++ within 20% of each other

#### 5.2 Shutdown Latency

**What to Measure:**
- Clean shutdown time
- Resource cleanup time
- Graceful termination under load

**Test Scenarios:**
```
Shutdown Triggers:
- SIGINT (Ctrl+C)
- SIGTERM
- After serving 1000 requests
- During active requests (graceful shutdown)

Instance Counts:
- 1, 10, 100 active instances
```

**Metrics:**
- Time from signal to exit (seconds)
- Number of resources leaked (should be 0)
- Unhandled requests (should be 0 for graceful)

**Success Criteria:**
- Target: <2 seconds for clean shutdown
- No memory leaks (valgrind)
- No panics or crashes

---

## Test Scenarios

### Scenario 1: Single Instance Simulation Workflow

**Description:** Simulate a typical FMU co-simulation workflow with a single instance.

**Steps:**
1. Start server with BouncingBall FMU
2. Create Liaison FMU client
3. Instantiate FMU (fmi3InstantiateCoSimulation)
4. Enter initialization mode
5. Set initial values (h=1.0, v=0.0)
6. Exit initialization mode
7. Run simulation loop (100 steps, dt=0.01):
   - Get outputs (h, v)
   - Perform doStep
   - Set inputs (if any)
8. Terminate FMU
9. Free instance
10. Shutdown server

**Metrics:**
- Total simulation time
- Time per step (avg, min, max, p50, p90, p99)
- Memory usage throughout
- Network bandwidth used

**Compare:**
- Rust server vs C++ server
- With direct FMU (no Liaison) as baseline

### Scenario 2: Multi-Instance Parallel Simulation

**Description:** Run multiple FMU instances in parallel to test scalability.

**Steps:**
1. Start server with BouncingBall FMU
2. Create 10 Liaison FMU clients
3. Instantiate 10 FMU instances (one per client)
4. Initialize all instances
5. Run all simulations in parallel (100 steps each)
6. Terminate and free all instances
7. Shutdown server

**Metrics:**
- Total wall-clock time
- Per-instance time (should be ~same as single instance)
- Server CPU usage (should scale with core count)
- Server memory usage (should scale linearly with instance count)
- Network bandwidth (should scale with instance count)

**Compare:**
- Rust vs C++ scalability characteristics
- Identify concurrency bottlenecks

### Scenario 3: High-Frequency Get/Set Operations

**Description:** Stress test with rapid get/set operations.

**Steps:**
1. Start server with test FMU
2. Create client and instantiate FMU
3. Initialize FMU
4. Perform 10,000 iterations of:
   - setFloat64 (10 values)
   - getFloat64 (10 values)
5. Terminate and cleanup

**Metrics:**
- Total time for 10,000 iterations
- Operations per second
- Latency distribution
- Memory allocation rate
- CPU usage

**Compare:**
- Rust vs C++ serialization efficiency
- FFI overhead impact

### Scenario 4: Large Binary Data Transfer

**Description:** Test performance with large binary payloads.

**Steps:**
1. Start server with test FMU supporting binary variables
2. Create client and instantiate FMU
3. Test binary transfers:
   - 1 KB binary data
   - 10 KB binary data
   - 100 KB binary data
   - 1 MB binary data
   - 10 MB binary data
4. Measure setBinary and getBinary separately

**Metrics:**
- Time per transfer (by size)
- Throughput (MB/s)
- Memory efficiency (allocations vs payload size)
- Network utilization

**Compare:**
- Rust vs C++ handling of large payloads
- Identify copy overhead

### Scenario 5: Long-Running Stability Test

**Description:** Test for memory leaks and performance degradation over time.

**Steps:**
1. Start server with BouncingBall FMU
2. Run continuous simulation for 24 hours:
   - Create instance
   - Run 1000 steps
   - Destroy instance
   - Repeat
3. Monitor throughout

**Metrics:**
- Memory usage over time (RSS, heap)
- Performance over time (any degradation?)
- CPU usage stability
- No crashes or errors

**Compare:**
- Rust vs C++ memory leak behavior
- Long-term stability

### Scenario 6: Cold Start vs Warm Performance

**Description:** Compare cold start (first request) vs warm (subsequent requests).

**Steps:**
1. Start fresh server
2. Measure first instantiation (cold)
3. Measure 100 subsequent instantiations (warm)
4. Compare latencies

**Metrics:**
- Cold start latency
- Warm latency (avg, min, max)
- JIT/caching effects (if any)

**Compare:**
- Rust vs C++ cold start overhead
- Steady-state performance

---

## Metrics and Success Criteria

### Primary Metrics

| Metric | Unit | Target (Rust) | Acceptance Criteria |
|--------|------|---------------|---------------------|
| **Memory** |
| Peak RSS (single instance) | MB | <30 MB | Within 20% of C++ |
| Peak RSS (100 instances) | MB | <500 MB | Within 20% of C++ |
| Memory growth (24h test) | MB/hour | 0 | No leaks detected |
| **Latency** |
| Median round-trip (localhost) | ms | <1 ms | Within 10% of C++ |
| p99 round-trip (localhost) | ms | <5 ms | Within 20% of C++ |
| Median round-trip (LAN) | ms | <5 ms | Within 10% of C++ |
| **Throughput** |
| Simple operations (localhost) | req/s | >1000 | Within 15% of C++ |
| Large data transfer | MB/s | >100 | Within 15% of C++ |
| **Startup** |
| Server startup (small FMU) | s | <5 s | Within 20% of C++ |
| First request latency | ms | <100 ms | Within 20% of C++ |
| **CPU** |
| Protobuf serialize (100B) | ns | <500 ns | Within 10% of C++ |
| FFI call overhead | ns | <100 ns | Within 20% of C++ |

### Secondary Metrics

| Metric | Purpose |
|--------|---------|
| CPU instructions per operation | Understand CPU efficiency |
| Cache miss rate | Identify cache-unfriendly code |
| Context switches per second | Measure threading overhead |
| Network packets per operation | Understand network efficiency |
| Heap allocations per operation | Identify allocation hotspots |
| Lock contention (if applicable) | Find synchronization bottlenecks |

### Comparison Methodology

**Statistical Significance:**
- Run each benchmark at least 10 times
- Calculate mean, median, standard deviation
- Use t-test to determine if differences are significant
- Report confidence intervals (95%)

**Variance Control:**
- Use same hardware for both implementations
- Run on dedicated machine (no other load)
- Disable CPU frequency scaling (performance governor)
- Use same network configuration
- Use same FMU files
- Measure at same time of day (minimize external factors)

**Normalization:**
- Report both absolute numbers and relative differences
- Normalize by baseline (e.g., direct FMU call)
- Account for measurement overhead

---

## Implementation Structure

### Directory Structure

```
liaison/
├── benches/                          # Benchmark implementations
│   ├── README.md                     # How to run benchmarks
│   │
│   ├── rust/                         # Rust-specific benchmarks
│   │   ├── Cargo.toml                # Benchmark dependencies
│   │   ├── benches/
│   │   │   ├── microbenchmarks.rs   # Criterion microbenchmarks
│   │   │   ├── memory.rs             # Memory usage benchmarks
│   │   │   ├── protobuf_bench.rs    # Serialization benchmarks
│   │   │   ├── ffi_bench.rs          # FFI overhead benchmarks
│   │   │   └── zenoh_bench.rs        # Zenoh operation benchmarks
│   │   └── tests/
│   │       ├── integration_bench.rs  # End-to-end benchmarks
│   │       ├── scenario_1.rs         # Single instance workflow
│   │       ├── scenario_2.rs         # Multi-instance parallel
│   │       ├── scenario_3.rs         # High-frequency operations
│   │       ├── scenario_4.rs         # Large binary transfers
│   │       └── scenario_5.rs         # Long-running stability
│   │
│   ├── cpp/                          # C++ benchmarks (for comparison)
│   │   ├── CMakeLists.txt
│   │   ├── microbenchmarks.cpp       # Google Benchmark micros
│   │   ├── memory_bench.cpp
│   │   ├── protobuf_bench.cpp
│   │   └── scenarios/
│   │       ├── scenario_1.cpp
│   │       ├── scenario_2.cpp
│   │       └── ...
│   │
│   ├── common/                       # Shared test data and scripts
│   │   ├── test_fmus/                # FMUs for benchmarking
│   │   │   ├── BouncingBall.fmu
│   │   │   ├── SmallFmu.fmu
│   │   │   ├── MediumFmu.fmu
│   │   │   └── LargeFmu.fmu
│   │   └── scripts/
│   │       ├── run_all_benchmarks.sh
│   │       ├── compare_results.py
│   │       ├── plot_results.py
│   │       └── validate_environment.sh
│   │
│   └── results/                      # Benchmark results
│       ├── baseline/                 # Known good results
│       ├── rust_YYYY-MM-DD/          # Timestamped runs
│       ├── cpp_YYYY-MM-DD/
│       └── comparison_YYYY-MM-DD/    # Side-by-side comparisons
│
└── docs/
    └── PERFORMANCE_BENCHMARKS_DESIGN.md  # This document
```

### Rust Benchmark Implementation

#### Using Criterion for Microbenchmarks

**benches/rust/Cargo.toml:**
```toml
[package]
name = "liaison-benchmarks"
version = "0.1.0"
edition = "2021"

[dependencies]
liaison-fmi = { path = "../../liaison-fmi" }
liaison-server = { path = "../../liaison-server" }
criterion = "0.5"
prost = "0.13"
zenoh = "1.0"
tokio = { version = "1.35", features = ["full"] }
tempfile = "3.10"

[dev-dependencies]
# For memory profiling
jemalloc-ctl = "0.5"

[[bench]]
name = "microbenchmarks"
harness = false

[[bench]]
name = "protobuf_bench"
harness = false

[[bench]]
name = "ffi_bench"
harness = false

[[bench]]
name = "zenoh_bench"
harness = false
```

**benches/rust/benches/microbenchmarks.rs:**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use liaison_server::proto;
use prost::Message;

fn protobuf_serialize_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("protobuf_serialize");

    // Empty message
    group.bench_function("empty_message", |b| {
        let msg = proto::GetVersionInput {};
        b.iter(|| {
            let mut buf = Vec::new();
            msg.encode(&mut buf).unwrap();
            black_box(buf);
        });
    });

    // Small message (getFloat64 with 10 values)
    group.bench_function("small_message", |b| {
        let msg = proto::GetFloat64Input {
            instance_index: 0,
            value_references: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            n_values: 10,
        };
        b.iter(|| {
            let mut buf = Vec::new();
            msg.encode(&mut buf).unwrap();
            black_box(buf);
        });
    });

    // Large message (getFloat64 with 1000 values)
    for size in [10, 100, 1000, 10000] {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("float64_array", size), &size, |b, &size| {
            let msg = proto::GetFloat64Input {
                instance_index: 0,
                value_references: (0..size).collect(),
                n_values: size as u64,
            };
            b.iter(|| {
                let mut buf = Vec::new();
                msg.encode(&mut buf).unwrap();
                black_box(buf);
            });
        });
    }

    group.finish();
}

fn protobuf_deserialize_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("protobuf_deserialize");

    // Prepare serialized messages
    let small_msg = proto::GetFloat64Input {
        instance_index: 0,
        value_references: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        n_values: 10,
    };
    let small_buf = small_msg.encode_to_vec();

    group.bench_function("small_message", |b| {
        b.iter(|| {
            let msg = proto::GetFloat64Input::decode(black_box(&small_buf[..])).unwrap();
            black_box(msg);
        });
    });

    group.finish();
}

criterion_group!(benches, protobuf_serialize_benchmark, protobuf_deserialize_benchmark);
criterion_main!(benches);
```

**benches/rust/benches/memory.rs:**
```rust
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

// Custom allocator to track allocations
struct TrackingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static DEALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc(layout);
        if !ret.is_null() {
            ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        DEALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
    }
}

#[global_allocator]
static GLOBAL: TrackingAllocator = TrackingAllocator;

fn get_allocated_bytes() -> usize {
    ALLOCATED.load(Ordering::SeqCst)
}

fn get_deallocated_bytes() -> usize {
    DEALLOCATED.load(Ordering::SeqCst)
}

fn reset_counters() {
    ALLOCATED.store(0, Ordering::SeqCst);
    DEALLOCATED.store(0, Ordering::SeqCst);
}

#[test]
fn measure_placeholder_memory() {
    use liaison_fmi::placeholder::Placeholder;

    reset_counters();
    let start_allocated = get_allocated_bytes();

    {
        let placeholder = Placeholder::new(
            std::ptr::null_mut(),
            None,
        ).unwrap();

        let after_creation = get_allocated_bytes();
        println!("Placeholder creation allocated: {} bytes",
                 after_creation - start_allocated);
    }

    let after_drop = get_deallocated_bytes();
    let leaked = get_allocated_bytes() - after_drop;
    println!("Leaked bytes: {}", leaked);
    assert_eq!(leaked, 0, "Memory leak detected!");
}
```

**benches/rust/tests/scenario_1.rs:**
```rust
use std::time::Instant;
use std::path::PathBuf;
use liaison_server::server::start_server;
use std::thread;
use std::time::Duration;

#[test]
fn single_instance_simulation_workflow() {
    // Setup
    let fmu_path = PathBuf::from("../common/test_fmus/BouncingBall.fmu");
    let responder_id = "test_benchmark".to_string();

    // Start server in background
    let server_handle = thread::spawn(move || {
        start_server(fmu_path, responder_id, None, None, false)
    });

    // Wait for server to be ready
    thread::sleep(Duration::from_secs(2));

    // Create Liaison FMU client (using FMPy or direct FMI calls)
    let start = Instant::now();

    // 1. Instantiate
    let instantiate_start = Instant::now();
    // ... FMI call ...
    let instantiate_time = instantiate_start.elapsed();

    // 2. Initialize
    let init_start = Instant::now();
    // ... FMI calls ...
    let init_time = init_start.elapsed();

    // 3. Simulation loop
    let sim_start = Instant::now();
    let mut step_times = Vec::new();

    for _ in 0..100 {
        let step_start = Instant::now();
        // ... doStep ...
        step_times.push(step_start.elapsed());
    }

    let sim_time = sim_start.elapsed();

    // 4. Terminate
    let term_start = Instant::now();
    // ... FMI calls ...
    let term_time = term_start.elapsed();

    let total_time = start.elapsed();

    // Report results
    println!("=== Scenario 1: Single Instance Workflow ===");
    println!("Total time: {:?}", total_time);
    println!("Instantiate: {:?}", instantiate_time);
    println!("Initialize: {:?}", init_time);
    println!("Simulation (100 steps): {:?}", sim_time);
    println!("  Avg step time: {:?}", sim_time / 100);
    println!("  Min step time: {:?}", step_times.iter().min().unwrap());
    println!("  Max step time: {:?}", step_times.iter().max().unwrap());
    println!("Terminate: {:?}", term_time);

    // Save results to JSON for comparison
    let results = serde_json::json!({
        "scenario": "single_instance_workflow",
        "implementation": "rust",
        "total_time_ms": total_time.as_millis(),
        "instantiate_time_ms": instantiate_time.as_millis(),
        "initialize_time_ms": init_time.as_millis(),
        "simulation_time_ms": sim_time.as_millis(),
        "avg_step_time_us": (sim_time.as_micros() / 100),
        "min_step_time_us": step_times.iter().min().unwrap().as_micros(),
        "max_step_time_us": step_times.iter().max().unwrap().as_micros(),
        "terminate_time_ms": term_time.as_millis(),
    });

    std::fs::write(
        "../../results/rust_scenario_1.json",
        serde_json::to_string_pretty(&results).unwrap()
    ).unwrap();

    // Cleanup
    // ... shutdown server ...
}
```

### C++ Benchmark Implementation

**benches/cpp/CMakeLists.txt:**
```cmake
cmake_minimum_required(VERSION 3.15)
project(liaison_benchmarks)

set(CMAKE_CXX_STANDARD 17)

# Find dependencies
find_package(benchmark REQUIRED)
find_package(Protobuf REQUIRED)

# Microbenchmarks
add_executable(microbenchmarks
    microbenchmarks.cpp
    protobuf_bench.cpp
)
target_link_libraries(microbenchmarks
    benchmark::benchmark
    ${Protobuf_LIBRARIES}
)

# Scenario benchmarks
add_executable(scenario_1 scenarios/scenario_1.cpp)
target_link_libraries(scenario_1
    # ... liaison libraries ...
)
```

**benches/cpp/microbenchmarks.cpp:**
```cpp
#include <benchmark/benchmark.h>
#include "fmi3.pb.h"

static void BM_ProtobufSerialize_SmallMessage(benchmark::State& state) {
    proto::fmi3GetFloat64InputMessage msg;
    msg.set_instance_index(0);
    for (int i = 0; i < 10; i++) {
        msg.add_value_references(i);
    }
    msg.set_n_value_references(10);

    for (auto _ : state) {
        std::string buf;
        msg.SerializeToString(&buf);
        benchmark::DoNotOptimize(buf);
    }

    state.SetBytesProcessed(state.iterations() * msg.ByteSizeLong());
}
BENCHMARK(BM_ProtobufSerialize_SmallMessage);

static void BM_ProtobufDeserialize_SmallMessage(benchmark::State& state) {
    proto::fmi3GetFloat64InputMessage msg;
    msg.set_instance_index(0);
    for (int i = 0; i < 10; i++) {
        msg.add_value_references(i);
    }
    msg.set_n_value_references(10);

    std::string buf;
    msg.SerializeToString(&buf);

    for (auto _ : state) {
        proto::fmi3GetFloat64InputMessage parsed;
        parsed.ParseFromString(buf);
        benchmark::DoNotOptimize(parsed);
    }

    state.SetBytesProcessed(state.iterations() * buf.size());
}
BENCHMARK(BM_ProtobufDeserialize_SmallMessage);

BENCHMARK_MAIN();
```

### Comparison Scripts

**benches/common/scripts/run_all_benchmarks.sh:**
```bash
#!/bin/bash
set -e

TIMESTAMP=$(date +%Y-%m-%d_%H-%M-%S)
RUST_RESULTS_DIR="../results/rust_${TIMESTAMP}"
CPP_RESULTS_DIR="../results/cpp_${TIMESTAMP}"
COMPARISON_DIR="../results/comparison_${TIMESTAMP}"

mkdir -p "$RUST_RESULTS_DIR" "$CPP_RESULTS_DIR" "$COMPARISON_DIR"

echo "=== Running Rust Benchmarks ==="
cd ../rust
cargo bench --bench microbenchmarks -- --save-baseline rust_baseline
cargo bench --bench protobuf_bench -- --save-baseline rust_baseline
cargo test --test scenario_1 --release -- --nocapture
cargo test --test scenario_2 --release -- --nocapture
# ... copy results ...

echo "=== Running C++ Benchmarks ==="
cd ../cpp
mkdir -p build && cd build
cmake -DCMAKE_BUILD_TYPE=Release ..
make -j$(nproc)
./microbenchmarks --benchmark_out="../results/cpp_micro.json" --benchmark_out_format=json
./scenario_1 > "../results/cpp_scenario_1.txt"
# ... copy results ...

echo "=== Comparing Results ==="
cd ../../scripts
python3 compare_results.py \
    --rust "$RUST_RESULTS_DIR" \
    --cpp "$CPP_RESULTS_DIR" \
    --output "$COMPARISON_DIR/comparison.html"

echo "=== Results Summary ==="
cat "$COMPARISON_DIR/summary.txt"
```

**benches/common/scripts/compare_results.py:**
```python
#!/usr/bin/env python3
import json
import sys
from pathlib import Path
import pandas as pd
import matplotlib.pyplot as plt

def load_results(rust_dir, cpp_dir):
    """Load benchmark results from both implementations."""
    rust_results = {}
    cpp_results = {}

    # Load Rust JSON results
    for json_file in Path(rust_dir).glob("*.json"):
        with open(json_file) as f:
            data = json.load(f)
            rust_results[json_file.stem] = data

    # Load C++ JSON results
    for json_file in Path(cpp_dir).glob("*.json"):
        with open(json_file) as f:
            data = json.load(f)
            cpp_results[json_file.stem] = data

    return rust_results, cpp_results

def compare_metric(rust_val, cpp_val, metric_name):
    """Compare a single metric and return relative difference."""
    if cpp_val == 0:
        return None

    diff_pct = ((rust_val - cpp_val) / cpp_val) * 100
    return {
        'metric': metric_name,
        'rust': rust_val,
        'cpp': cpp_val,
        'diff_pct': diff_pct,
        'rust_faster': diff_pct < 0
    }

def generate_report(rust_results, cpp_results, output_dir):
    """Generate HTML report with comparison tables and charts."""
    comparisons = []

    # Compare scenario 1 results
    if 'scenario_1' in rust_results and 'scenario_1' in cpp_results:
        r1 = rust_results['scenario_1']
        c1 = cpp_results['scenario_1']

        comparisons.append(compare_metric(
            r1['total_time_ms'], c1['total_time_ms'],
            'Scenario 1: Total Time (ms)'
        ))
        comparisons.append(compare_metric(
            r1['avg_step_time_us'], c1['avg_step_time_us'],
            'Scenario 1: Avg Step Time (μs)'
        ))

    # Create DataFrame
    df = pd.DataFrame(comparisons)

    # Generate HTML report
    html = f"""
    <html>
    <head><title>Liaison Performance Comparison</title></head>
    <body>
    <h1>Liaison FMI: Rust vs C++ Performance</h1>
    <h2>Summary</h2>
    {df.to_html()}
    <h2>Charts</h2>
    <img src="comparison_chart.png">
    </body>
    </html>
    """

    output_path = Path(output_dir) / "comparison.html"
    with open(output_path, 'w') as f:
        f.write(html)

    # Generate chart
    fig, ax = plt.subplots(figsize=(12, 6))
    metrics = df['metric'].tolist()
    x = range(len(metrics))

    ax.bar([i - 0.2 for i in x], df['rust'], width=0.4, label='Rust', alpha=0.8)
    ax.bar([i + 0.2 for i in x], df['cpp'], width=0.4, label='C++', alpha=0.8)

    ax.set_xlabel('Metric')
    ax.set_ylabel('Value')
    ax.set_title('Rust vs C++ Performance Comparison')
    ax.set_xticks(x)
    ax.set_xticklabels(metrics, rotation=45, ha='right')
    ax.legend()
    ax.grid(axis='y', alpha=0.3)

    plt.tight_layout()
    plt.savefig(Path(output_dir) / "comparison_chart.png", dpi=150)

    # Generate summary text
    summary = f"""
    Liaison Performance Benchmark Summary
    ======================================

    Total Metrics Compared: {len(comparisons)}
    Rust Faster: {sum(1 for c in comparisons if c['rust_faster'])}
    C++ Faster: {sum(1 for c in comparisons if not c['rust_faster'])}

    Average Difference: {df['diff_pct'].mean():.2f}%
    Max Rust Advantage: {df['diff_pct'].min():.2f}%
    Max C++ Advantage: {df['diff_pct'].max():.2f}%
    """

    summary_path = Path(output_dir) / "summary.txt"
    with open(summary_path, 'w') as f:
        f.write(summary)

    print(summary)

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument('--rust', required=True, help='Rust results directory')
    parser.add_argument('--cpp', required=True, help='C++ results directory')
    parser.add_argument('--output', required=True, help='Output directory')
    args = parser.parse_args()

    rust_results, cpp_results = load_results(args.rust, args.cpp)
    generate_report(rust_results, cpp_results, args.output)
```

---

## Running Benchmarks

### Prerequisites

**System Configuration:**
```bash
# Disable CPU frequency scaling for consistent results
sudo cpupower frequency-set --governor performance

# Disable swap to avoid memory thrashing
sudo swapoff -a

# Set process priority
sudo nice -n -20 <benchmark_command>

# Disable hyperthreading (optional, for more consistent results)
echo off | sudo tee /sys/devices/system/cpu/smt/control
```

**Install Dependencies:**
```bash
# Rust
cd benches/rust
cargo build --release

# C++
cd benches/cpp
mkdir build && cd build
cmake -DCMAKE_BUILD_TYPE=Release ..
make -j$(nproc)

# Python (for analysis)
pip install pandas matplotlib seaborn numpy scipy
```

### Running Individual Benchmarks

**Rust Microbenchmarks:**
```bash
cd benches/rust

# Run all microbenchmarks
cargo bench

# Run specific benchmark
cargo bench --bench protobuf_bench

# Save baseline for comparison
cargo bench -- --save-baseline rust_v1

# Compare against baseline
cargo bench -- --baseline rust_v1

# Generate detailed report
cargo bench -- --verbose
```

**Rust Scenario Tests:**
```bash
# Run scenario 1 (single instance)
cargo test --test scenario_1 --release -- --nocapture

# Run all scenarios
cargo test --tests --release -- --nocapture

# With memory profiling
CARGO_PROFILE_RELEASE_DEBUG=true cargo test --test scenario_1 --release
valgrind --tool=massif --massif-out-file=massif.out ./target/release/scenario_1
ms_print massif.out
```

**C++ Benchmarks:**
```bash
cd benches/cpp/build

# Run microbenchmarks
./microbenchmarks

# With JSON output
./microbenchmarks --benchmark_out=results.json --benchmark_out_format=json

# Run specific benchmark filter
./microbenchmarks --benchmark_filter=Protobuf.*

# With profiling
perf record -g ./microbenchmarks
perf report
```

**Memory Profiling:**
```bash
# Rust - using valgrind
valgrind --leak-check=full --show-leak-kinds=all \
    ./target/release/liaison serve test.fmu test_id

# Rust - using heaptrack
heaptrack ./target/release/liaison serve test.fmu test_id
heaptrack_gui heaptrack.liaison.*.zst

# C++ - using valgrind
valgrind --leak-check=full --show-leak-kinds=all \
    ./build/liaison --serve test.fmu test_id

# Both - RSS monitoring
while true; do
    ps aux | grep liaison | grep -v grep | awk '{print $6}'
    sleep 1
done > memory_usage.txt
```

**Network Latency Measurement:**
```bash
# Start server
./target/release/liaison serve test.fmu test_id

# Run latency test (separate terminal)
cd benches/rust
cargo test --test network_latency --release -- --nocapture

# Capture packets for analysis
sudo tcpdump -i lo -w liaison.pcap port 7447
```

### Running Complete Benchmark Suite

**Full Automated Run:**
```bash
cd benches/common/scripts
./run_all_benchmarks.sh
```

**Manual Step-by-Step:**
```bash
# 1. Validate environment
./validate_environment.sh

# 2. Run Rust benchmarks
cd ../../rust
./run_rust_benchmarks.sh

# 3. Run C++ benchmarks
cd ../../cpp
./run_cpp_benchmarks.sh

# 4. Compare results
cd ../../common/scripts
python3 compare_results.py \
    --rust ../../results/rust_latest \
    --cpp ../../results/cpp_latest \
    --output ../../results/comparison_latest

# 5. View report
firefox ../../results/comparison_latest/comparison.html
```

---

## Results Analysis

### Output Format

**Criterion Output (Rust):**
```
protobuf_serialize/empty_message
                        time:   [45.231 ns 45.567 ns 45.923 ns]
                        change: [-2.3421% -1.5234% -0.7891%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

protobuf_serialize/small_message
                        time:   [123.45 ns 124.78 ns 126.23 ns]
                        thrpt:  [79.234 Melem/s 80.134 Melem/s 81.023 Melem/s]
```

**Google Benchmark Output (C++):**
```
Run on (8 X 3600 MHz CPU s)
CPU Caches:
  L1 Data 32 KiB (x4)
  L1 Instruction 32 KiB (x4)
  L2 Unified 256 KiB (x4)
  L3 Unified 8192 KiB (x1)

----------------------------------------------------------------------
Benchmark                            Time             CPU   Iterations
----------------------------------------------------------------------
BM_ProtobufSerialize_SmallMessage  125 ns          125 ns      5592405
BM_ProtobufDeserialize_SmallMessage 98 ns           98 ns      7142857
```

**Scenario Test Output:**
```json
{
  "scenario": "single_instance_workflow",
  "implementation": "rust",
  "timestamp": "2025-11-09T12:34:56Z",
  "system": {
    "os": "Linux 5.15.0",
    "cpu": "Intel Xeon E-2176M @ 2.70GHz",
    "ram": "32 GB",
    "cores": 6
  },
  "results": {
    "total_time_ms": 1234,
    "instantiate_time_ms": 123,
    "initialize_time_ms": 45,
    "simulation_time_ms": 980,
    "avg_step_time_us": 9800,
    "min_step_time_us": 8500,
    "max_step_time_us": 15000,
    "p50_step_time_us": 9600,
    "p90_step_time_us": 11200,
    "p99_step_time_us": 13500,
    "terminate_time_ms": 86,
    "memory_peak_mb": 28,
    "memory_avg_mb": 25
  }
}
```

### Interpretation Guidelines

**Statistical Significance:**
- Difference <5%: Likely noise, not significant
- Difference 5-10%: Potentially significant, needs more samples
- Difference >10%: Likely significant, investigate cause

**Performance Categories:**
- Excellent: Rust within 5% of C++
- Good: Rust within 10% of C++
- Acceptable: Rust within 20% of C++
- Needs Investigation: Rust >20% slower than C++

**Common Patterns:**

1. **Rust Faster Scenarios:**
   - Memory safety checks eliminated by optimizer
   - LLVM optimizations (tail call, inlining)
   - Zero-cost abstractions
   - Better cache locality from RAII

2. **C++ Faster Scenarios:**
   - Manual memory control (e.g., custom allocators)
   - Fine-tuned assembly
   - Less abstraction overhead

3. **Equal Performance:**
   - Network-bound operations (Zenoh dominates)
   - FFI calls (same underlying ABI)
   - Protobuf serialization (similar codegen)

### Visualization Examples

**Latency Distribution:**
```python
import matplotlib.pyplot as plt
import numpy as np

rust_latencies = [...]  # Load from results
cpp_latencies = [...]

fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12, 5))

# Histogram
ax1.hist(rust_latencies, bins=50, alpha=0.7, label='Rust')
ax1.hist(cpp_latencies, bins=50, alpha=0.7, label='C++')
ax1.set_xlabel('Latency (μs)')
ax1.set_ylabel('Frequency')
ax1.set_title('Latency Distribution')
ax1.legend()
ax1.grid(True, alpha=0.3)

# CDF
rust_sorted = np.sort(rust_latencies)
cpp_sorted = np.sort(cpp_latencies)
rust_cdf = np.arange(1, len(rust_sorted)+1) / len(rust_sorted)
cpp_cdf = np.arange(1, len(cpp_sorted)+1) / len(cpp_sorted)

ax2.plot(rust_sorted, rust_cdf, label='Rust', linewidth=2)
ax2.plot(cpp_sorted, cpp_cdf, label='C++', linewidth=2)
ax2.set_xlabel('Latency (μs)')
ax2.set_ylabel('Cumulative Probability')
ax2.set_title('Latency CDF')
ax2.legend()
ax2.grid(True, alpha=0.3)

plt.tight_layout()
plt.savefig('latency_comparison.png', dpi=150)
```

**Memory Usage Over Time:**
```python
import pandas as pd
import matplotlib.pyplot as plt

# Load memory samples
rust_mem = pd.read_csv('rust_memory.csv')
cpp_mem = pd.read_csv('cpp_memory.csv')

plt.figure(figsize=(12, 6))
plt.plot(rust_mem['time'], rust_mem['rss_mb'], label='Rust', linewidth=2)
plt.plot(cpp_mem['time'], cpp_mem['rss_mb'], label='C++', linewidth=2)
plt.xlabel('Time (seconds)')
plt.ylabel('RSS (MB)')
plt.title('Memory Usage Over Time (24h Stability Test)')
plt.legend()
plt.grid(True, alpha=0.3)
plt.savefig('memory_stability.png', dpi=150)
```

**Throughput vs Latency Trade-off:**
```python
import matplotlib.pyplot as plt

throughputs = [100, 200, 500, 1000, 2000, 5000]  # req/s
rust_latencies = [...]  # p99 latencies at each throughput
cpp_latencies = [...]

plt.figure(figsize=(10, 6))
plt.plot(throughputs, rust_latencies, 'o-', label='Rust', linewidth=2, markersize=8)
plt.plot(throughputs, cpp_latencies, 's-', label='C++', linewidth=2, markersize=8)
plt.xlabel('Throughput (requests/second)')
plt.ylabel('p99 Latency (ms)')
plt.title('Throughput vs Latency Trade-off')
plt.legend()
plt.grid(True, alpha=0.3)
plt.xscale('log')
plt.yscale('log')
plt.savefig('throughput_latency.png', dpi=150)
```

---

## Fair Comparison Guidelines

### Environment Consistency

**Hardware:**
- Use identical hardware for both implementations
- Document CPU, RAM, disk type, network interface
- Run on bare metal (not VM) for best consistency
- Disable Turbo Boost for reproducibility

**Software:**
- Same OS version and kernel
- Same compiler optimization levels (-O3 / --release)
- Same protobuf version (wire format compatible)
- Same Zenoh version
- Same FMU files

**Configuration:**
- Same Zenoh configuration (endpoints, QoS, etc.)
- Same network settings (MTU, TCP window, etc.)
- Same logging level (or disabled)
- Same debug/release mode

### Measurement Fairness

**Avoid Bias:**
- Randomize test order (Rust first vs C++ first)
- Run multiple iterations (at least 10)
- Discard outliers (using statistical methods)
- Warm up before measuring (JIT, caches)
- Measure both cold start and steady state

**Overhead Accounting:**
- Subtract measurement overhead (timing cost)
- Account for test harness differences
- Isolate benchmark from test framework
- Use black_box / DoNotOptimize to prevent over-optimization

**Statistical Rigor:**
- Report mean, median, std dev, percentiles
- Use appropriate statistical tests (t-test, Mann-Whitney)
- Calculate confidence intervals
- Check for normal distribution (Q-Q plot)
- Account for autocorrelation in time series

### Apples-to-Apples Comparisons

**What to Compare:**
✅ Same FMI functions (getFloat64 vs getFloat64)
✅ Same message sizes
✅ Same network configuration
✅ Same workload patterns
✅ Same success criteria (both correct results)

**What NOT to Compare:**
❌ Debug build vs Release build
❌ Different FMU files
❌ Different Zenoh modes (peer vs client)
❌ With logging vs without logging
❌ Different measurement tools (timing precision)

### Optimization Levels

**Rust:**
```toml
[profile.release]
opt-level = 3          # Maximum optimizations
lto = true             # Link-time optimization
codegen-units = 1      # Better optimization
strip = false          # Keep symbols for profiling
debug = false          # No debug info
```

**C++:**
```cmake
set(CMAKE_BUILD_TYPE Release)
set(CMAKE_CXX_FLAGS_RELEASE "-O3 -march=native -flto")
```

**Ensure Equivalence:**
- Both using LTO
- Both using maximum optimization (-O3 / opt-level=3)
- Both stripped (or both with symbols)
- No platform-specific asm in C++ that Rust can't match

### Accounting for Implementation Differences

**Legitimate Differences:**
- Rust bounds checking (safety vs speed trade-off)
- C++ manual memory management (can be faster, but unsafe)
- Different libraries (e.g., protobuf-c++ vs prost)

**How to Handle:**
- Document the difference
- Measure impact separately
- Provide both "fair" and "optimized" numbers
- Example: "Rust with bounds checking: 125ns, Rust unsafe (no bounds check): 100ns"

**Report Format:**
```
Operation: Protobuf Deserialize (100 byte message)
Rust (safe):     125 ns ± 5 ns
Rust (unsafe):   100 ns ± 4 ns
C++:              98 ns ± 3 ns

Interpretation:
- Rust safe version is 27% slower due to bounds checking
- Rust unsafe version is within 2% of C++ (not significant)
- For production, recommend safe version (negligible overhead in context)
```

### Disclosure of Limitations

**Be Transparent:**
- Document any known performance issues
- Explain architectural differences
- Note unoptimized code paths
- Identify future optimization opportunities

**Example:**
```
Known Limitations:
1. Rust version does not yet use custom allocator (planned)
2. C++ version uses manual SIMD for large arrays (Rust uses auto-vectorization)
3. Both implementations could benefit from zero-copy deserialization (future work)
```

### Reproducibility

**Provide:**
- Complete benchmark source code
- Build instructions
- Exact dependency versions (Cargo.lock, package-lock.json)
- Environment validation script
- Sample FMU files
- Expected results (baseline)

**Documentation:**
```markdown
## Reproducing Benchmarks

### System Requirements
- CPU: x86_64 with AVX2
- RAM: 16 GB minimum
- OS: Ubuntu 22.04 LTS
- Disk: SSD (NVMe recommended)

### Setup
1. Install dependencies: `./install_deps.sh`
2. Validate environment: `./validate_environment.sh`
3. Configure system: `./configure_system.sh`

### Run
```bash
./run_all_benchmarks.sh
```

### Expected Results
See `baseline/` directory for reference results.
Variance: <5% from baseline on similar hardware.
```

---

## Appendix: Example Benchmark Results

### Sample Output

```
=== Liaison Performance Benchmark Report ===
Date: 2025-11-09
Hardware: Intel Xeon E-2176M @ 2.70GHz, 32 GB RAM
OS: Ubuntu 22.04 LTS, Kernel 5.15.0

=== Microbenchmarks ===

Protobuf Serialize (100 byte message):
  Rust:  118 ns ± 3 ns
  C++:   121 ns ± 4 ns
  Winner: Rust (2.5% faster, not significant)

Protobuf Deserialize (100 byte message):
  Rust:  95 ns ± 2 ns
  C++:   92 ns ± 2 ns
  Winner: C++ (3.3% faster, not significant)

FFI Call Overhead:
  Rust:  68 ns ± 5 ns
  C++:   N/A (baseline)
  Overhead: 68 ns

Zenoh Query (localhost, 100 byte):
  Rust:  842 μs ± 23 μs
  C++:   856 μs ± 28 μs
  Winner: Rust (1.6% faster, not significant)

=== Scenario 1: Single Instance Workflow ===

Total Time:
  Rust:  1.234 s
  C++:   1.198 s
  Diff:  +3.0% (Rust slower)

Avg Step Time:
  Rust:  9.8 ms
  C++:   9.5 ms
  Diff:  +3.2% (Rust slower)

Memory Peak:
  Rust:  28 MB
  C++:   32 MB
  Winner: Rust (12.5% less memory)

=== Scenario 2: Multi-Instance (10 instances) ===

Total Time:
  Rust:  2.456 s
  C++:   2.401 s
  Diff:  +2.3% (Rust slower)

Memory Peak:
  Rust:  142 MB
  C++:   168 MB
  Winner: Rust (15.5% less memory)

Scalability (vs single instance):
  Rust:  1.99x (near linear)
  C++:   2.00x (near linear)

=== Summary ===

Performance: C++ has slight edge (2-3% faster on average)
Memory: Rust uses 12-15% less memory
Statistical Significance: Most differences within noise margin
Verdict: Equivalent performance, Rust has memory advantage

Recommendation: Rust implementation is production-ready from
performance perspective. Memory safety benefits outweigh minor
performance differences.
```

### Interpreting Results

**Key Questions:**

1. **Are the differences significant?**
   - Use t-test or confidence intervals
   - <5% likely noise, >10% likely real

2. **What causes the differences?**
   - Profile both implementations
   - Identify hot paths
   - Compare generated assembly

3. **Do the differences matter?**
   - In context of total simulation time
   - Network latency often dominates
   - 2-3% difference is negligible for most users

4. **Where to optimize?**
   - Focus on >10% differences
   - Optimize hot paths (80/20 rule)
   - Consider memory vs speed trade-offs

---

## Conclusion

This performance benchmark framework provides comprehensive, fair, and reproducible measurements of the Liaison FMI Rust implementation compared to the C++ version. Key principles:

1. **Comprehensive Coverage**: Measure all aspects (CPU, memory, network, latency, throughput)
2. **Statistical Rigor**: Multiple runs, outlier removal, confidence intervals
3. **Fairness**: Identical hardware, config, workloads, optimization levels
4. **Transparency**: Document limitations, implementation differences
5. **Reproducibility**: Complete source, exact dependencies, validation scripts
6. **Context**: Interpret numbers in context of real-world usage

**Expected Outcome:**

Based on the migration design and implementation quality, we expect:
- Rust and C++ to have equivalent performance (within 5-10%)
- Rust to have lower memory usage due to RAII
- Network latency to dominate over language differences
- No performance regressions from migration

**Next Steps:**

1. Implement benchmarks following this design
2. Establish baseline results
3. Run comparison measurements
4. Analyze and document findings
5. Optimize identified bottlenecks
6. Integrate into CI/CD for regression detection

---

**Document Version:** 1.0
**Last Updated:** 2025-11-09
**Author:** Liaison Development Team
**Status:** Design Phase - Ready for Implementation
