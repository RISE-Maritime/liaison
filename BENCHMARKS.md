# Liaison FMI Rust Benchmarks

This document describes the comprehensive benchmark suite for the Liaison FMI Rust implementation, covering both the client library (`liaison-fmi`) and server (`liaison-server`).

## Overview

The benchmarks are built using the [Criterion.rs](https://github.com/bheisler/criterion.rs) framework, which provides:
- Statistical analysis of performance
- HTML reports with graphs
- Detection of performance regressions
- Multiple iteration sizes for robust measurements

## Benchmark Structure

### Client Library Benchmarks (`liaison-fmi/benches/`)

Location: `/workspace/liaison-fmi/benches/client_benchmarks.rs`

#### 1. Protobuf Serialization/Deserialization
- **Message Encoding**: Tests encoding performance for various message types
  - Simple messages (InstanceMessage, StatusMessage)
  - Complex messages (DoStepMessage, InstantiateCoSimulation)
  - Variable-sized messages (Float64, Int32 arrays with 1, 10, 100, 1000 elements)
  - String-heavy messages (LogMessage)

- **Message Decoding**: Tests decoding performance for the same message types

- **Round-trip Operations**: Tests full encode + decode cycles

#### 2. Type Conversions
- **Status Conversions**: Benchmarks for converting between:
  - `proto::Status` → `fmi3Status`
  - `fmi3Status` → `proto::Status`
  - `i32` → `fmi3Status` (including invalid values)

- **Round-trip Conversions**: Tests bidirectional conversion performance

- **Batch Conversions**: Tests converting arrays of status values

#### 3. Message Creation and Manipulation
- **Simple Messages**: Creation of basic protobuf messages
- **Complex Messages**: Creation with nested structures and vectors
- **Vector-based Messages**: Testing with varying data sizes (1-1000 elements)
- **String Operations**: Messages with different string lengths (10-1000 chars)

#### 4. Encoded Length Calculations
- Tests the `encoded_len()` operation for size estimation
- Varies message complexity and data sizes

### Server Benchmarks (`liaison-server/benches/`)

Location: `/workspace/liaison-server/benches/server_benchmarks.rs`

#### 1. Instance Manager Operations
- **Add Operations**:
  - Single instance addition
  - Batch additions (10, 100, 1000 instances)

- **Get Operations**:
  - Successful retrieval
  - Not found scenarios
  - Invalid index handling
  - Random access patterns
  - Sequential access patterns

- **Remove Operations**:
  - Single removal (success and failure cases)
  - Batch removals (10, 100, 1000 instances)

- **Query Operations**:
  - `instance_count()` with varying instance counts
  - `contains_instance()` for existing and non-existing instances

- **Mixed Workload**:
  - Realistic simulation patterns (add → read → remove)
  - Small and large scale scenarios

- **Thread Safety**:
  - Manager cloning overhead
  - Concurrent access simulation

#### 2. Protobuf Operations (Server-side)
- Instance message encode/decode
- Status message round-trips
- Large Float64 messages (varying sizes: 1-1000 elements)

#### 3. FMU Creator Operations
- **JSON Configuration**:
  - Parsing simple configs
  - Parsing complex configs with TLS settings
  - Serialization (normal and pretty-printed)
  - Field extraction
  - Config modification

- **Path Operations**:
  - Model name extraction
  - Certificate filename extraction
  - Output path construction
  - Library path formatting

#### 4. Error Handling
- Error creation for different variants
- Error message formatting
- Result pattern matching (Ok and Err cases)

#### 5. Utility Operations
- **String Operations**:
  - Key expression formatting
  - String cloning (varying lengths: 10-1000 chars)
  - `&str` to `String` conversion

- **Vector Operations**:
  - Vector creation with capacity
  - Vector population
  - Float vector initialization

## Running Benchmarks

### Run All Benchmarks

```bash
# Run all benchmarks in the workspace
cargo bench

# Run only client library benchmarks
cargo bench --package liaison-fmi

# Run only server benchmarks
cargo bench --package liaison-server
```

### Run Specific Benchmark Groups

```bash
# Run protobuf benchmarks only (client)
cargo bench --package liaison-fmi --bench client_benchmarks -- protobuf

# Run instance manager benchmarks only (server)
cargo bench --package liaison-server --bench server_benchmarks -- instance_manager

# Run conversion benchmarks only (client)
cargo bench --package liaison-fmi --bench client_benchmarks -- conversion
```

### Run Specific Benchmarks

```bash
# Run a specific benchmark function
cargo bench --package liaison-fmi -- bench_protobuf_encode

# Run benchmarks matching a pattern
cargo bench --package liaison-server -- "instance_manager_add"
```

### Generate HTML Reports

Criterion automatically generates HTML reports in:
- `target/criterion/` directory
- Open `target/criterion/report/index.html` in a browser

### Save Baseline for Comparison

```bash
# Save current performance as baseline
cargo bench -- --save-baseline main

# Compare against baseline
cargo bench -- --baseline main

# List available baselines
ls target/criterion/*/base/
```

## Benchmark Configuration

The benchmarks use default Criterion settings:
- **Sample size**: 100 iterations
- **Measurement time**: 5 seconds per benchmark
- **Warm-up time**: 3 seconds
- **Significance level**: 0.05
- **Noise threshold**: 0.01

These can be customized in the benchmark files if needed.

## Interpreting Results

Criterion provides several statistics:
- **Mean**: Average execution time
- **Std Dev**: Standard deviation of measurements
- **Median**: Middle value of all measurements
- **MAD**: Median Absolute Deviation (robust measure of variance)

### Performance Guidelines

Based on the benchmark structure, typical expected ranges:

**Client Library:**
- Simple message encoding: < 100 ns
- Simple message decoding: < 200 ns
- Complex message encoding: < 500 ns
- Status conversions: < 10 ns
- Large array operations (1000 elements): < 10 μs

**Server:**
- Instance manager operations: < 100 ns
- JSON parsing (simple): < 1 μs
- JSON parsing (complex): < 5 μs
- Vector operations: Linear with size

### Regression Detection

Criterion automatically detects performance regressions:
- **Green**: Performance improved
- **Yellow**: No significant change
- **Red**: Performance regression detected

## Continuous Integration

To run benchmarks in CI:

```yaml
# .github/workflows/benchmarks.yml
- name: Run benchmarks
  run: cargo bench --workspace -- --output-format bencher
```

For tracking over time, consider:
- [Bencher](https://bencher.dev/)
- [Criterion-Table](https://github.com/nu11ptr/criterion-table)
- Custom tooling with `--output-format json`

## Troubleshooting

### Benchmarks Take Too Long

Reduce sample size or measurement time:
```rust
group.sample_size(10);
group.measurement_time(std::time::Duration::from_secs(2));
```

### Noisy Results

- Close other applications
- Disable CPU frequency scaling: `sudo cpupower frequency-set --governor performance`
- Run on a dedicated machine
- Increase sample size for more stable results

### Memory Issues with Large Benchmarks

Use `iter_batched` with `BatchSize::SmallInput` or `LargeInput` appropriately:
```rust
b.iter_batched(
    || expensive_setup(),
    |data| test_function(data),
    criterion::BatchSize::SmallInput,
);
```

## Adding New Benchmarks

### Client Library Benchmark Template

```rust
fn bench_new_operation(c: &mut Criterion) {
    let mut group = c.benchmark_group("operation_name");

    group.bench_function("specific_case", |b| {
        b.iter(|| {
            let result = operation(black_box(input));
            black_box(result)
        });
    });

    group.finish();
}

// Add to criterion_group! macro
```

### Server Benchmark Template

```rust
fn bench_new_server_operation(c: &mut Criterion) {
    let mut group = c.benchmark_group("server_operation");

    // Use iter_batched for setup/teardown
    group.bench_function("case", |b| {
        b.iter_batched(
            || setup(),
            |data| test(black_box(data)),
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}
```

## Best Practices

1. **Use `black_box`**: Prevent compiler optimizations from eliminating code
   ```rust
   black_box(value)  // Prevents optimization
   ```

2. **Separate Setup from Measurement**: Use `iter_batched` for expensive setup
   ```rust
   b.iter_batched(setup, test, BatchSize::SmallInput)
   ```

3. **Test Multiple Sizes**: Use parametric benchmarks for scalability analysis
   ```rust
   for size in [10, 100, 1000].iter() {
       group.bench_with_input(BenchmarkId::new("op", size), size, |b, &size| {
           // benchmark code
       });
   }
   ```

4. **Set Throughput**: For operations on multiple elements
   ```rust
   group.throughput(Throughput::Elements(count as u64));
   ```

5. **Group Related Benchmarks**: Use benchmark groups for organization
   ```rust
   let mut group = c.benchmark_group("related_operations");
   ```

## Performance Optimization Tips

Based on benchmark results, consider:

1. **Protobuf**: Pre-allocate buffers with `Vec::with_capacity(msg.encoded_len())`
2. **Status Conversions**: These are extremely fast; inline optimization is effective
3. **Instance Manager**: Lock contention is minimal for typical workloads
4. **JSON Operations**: Reuse parsers/serializers when possible
5. **Vector Operations**: Pre-allocate with known capacity

## References

- [Criterion.rs User Guide](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [FMI 3.0 Specification](https://fmi-standard.org/)

## Benchmark Maintenance

- Run benchmarks before major releases
- Update baselines after intentional performance changes
- Document significant performance improvements/regressions
- Review HTML reports for trends and outliers
- Keep benchmarks up-to-date with API changes
