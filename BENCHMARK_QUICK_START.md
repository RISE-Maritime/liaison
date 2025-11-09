# Benchmark Quick Start Guide

## Installation

No additional installation required. Benchmarks use Criterion which is already added as a dev-dependency.

## Quick Commands

### Run Everything
```bash
cargo bench
```

### Run Client Library Benchmarks Only
```bash
cargo bench -p liaison-fmi
```

### Run Server Benchmarks Only
```bash
cargo bench -p liaison-server
```

### Run Specific Benchmark Categories

**Client Library:**
```bash
# Protobuf operations
cargo bench -p liaison-fmi -- protobuf

# Type conversions
cargo bench -p liaison-fmi -- conversion

# Message creation
cargo bench -p liaison-fmi -- message
```

**Server:**
```bash
# Instance manager
cargo bench -p liaison-server -- instance_manager

# JSON operations
cargo bench -p liaison-server -- json

# Protobuf operations
cargo bench -p liaison-server -- protobuf
```

### View Results

Open the HTML report:
```bash
# Linux/Mac
open target/criterion/report/index.html

# Or just navigate to:
# file:///path/to/workspace/target/criterion/report/index.html
```

### Compare Performance

```bash
# Save baseline
cargo bench -- --save-baseline before-optimization

# Make changes...

# Compare against baseline
cargo bench -- --baseline before-optimization
```

## What Gets Benchmarked

### liaison-fmi (Client Library)
- ✅ Protobuf message encoding (serialize)
- ✅ Protobuf message decoding (deserialize)
- ✅ Round-trip encode/decode operations
- ✅ Status type conversions (Proto ↔ FMI ↔ i32)
- ✅ Message creation with varying data sizes
- ✅ String handling in messages
- ✅ Vector operations (1 to 1000 elements)

### liaison-server (Server)
- ✅ Instance manager add/get/remove operations
- ✅ Instance manager with 10-1000 instances
- ✅ Thread-safe concurrent access
- ✅ JSON config parsing and serialization
- ✅ Path extraction and manipulation
- ✅ Error creation and handling
- ✅ String and vector utilities
- ✅ Protobuf operations

## Typical Results

**Fast operations (< 100 ns):**
- Status conversions
- Instance manager get/add
- Simple message creation

**Medium operations (100 ns - 1 μs):**
- Simple protobuf encode/decode
- Small JSON parsing
- Vector operations < 100 elements

**Slower operations (> 1 μs):**
- Large protobuf messages (1000+ elements)
- Complex JSON with TLS config
- Large vector operations

## Tips for Better Results

1. **Close unnecessary applications** - reduce CPU noise
2. **Disable CPU frequency scaling** - consistent clock speeds
3. **Run multiple times** - verify consistency
4. **Use release mode** - benchmarks automatically use optimized builds

## Troubleshooting

**"No benchmarks to run"**
- Ensure you're in the workspace root directory
- Check that benchmark files exist in `*/benches/`

**"Command not found: cargo"**
- Install Rust: https://rustup.rs/

**Benchmarks take too long**
- This is normal! Complete benchmark suite takes 10-20 minutes
- Run specific benchmarks instead of all at once

## Example Output

```
Benchmarking protobuf_encode/instance_message
Benchmarking protobuf_encode/instance_message: Warming up for 3.0000 s
Benchmarking protobuf_encode/instance_message: Collecting 100 samples in estimated 5.0000 s (1000000 iterations)
Benchmarking protobuf_encode/instance_message: Analyzing
protobuf_encode/instance_message
                        time:   [45.234 ns 45.789 ns 46.412 ns]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
```

This means encoding a simple instance message takes ~46 nanoseconds on average.

## Next Steps

- Read `/workspace/BENCHMARKS.md` for detailed documentation
- View HTML reports in `target/criterion/report/`
- Add custom benchmarks for your specific use cases
- Set up CI/CD to track performance over time
