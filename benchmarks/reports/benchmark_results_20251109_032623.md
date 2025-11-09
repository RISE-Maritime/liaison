# Benchmark Results Summary

**Generated:** 2025-11-09 03:26:23

**Total Benchmarks:** 20


## Binary Size Benchmarks

| Benchmark | C++ | Rust | Difference | Winner |
|-----------|-----|------|------------|--------|
| client_library | 5.10 MB | 13.00 MB | +154.9% | CPP |
| server_binary | 8.20 MB | 15.00 MB | +82.9% | CPP |

## Build Time Benchmarks

| Benchmark | C++ | Rust | Difference | Winner |
|-----------|-----|------|------------|--------|
| clean_build | 35.00 seconds | 85.00 seconds | +142.9% | CPP |
| incremental_build | 6.00 seconds | 5.00 seconds | -16.7% | RUST |

## Latency Benchmarks

| Benchmark | C++ | Rust | Difference | Winner |
|-----------|-----|------|------------|--------|
| fmi3DoStep | 145.00 µs | 148.00 µs | +2.1% | CPP |
| fmi3EnterInitializationMode | 156.00 µs | 159.00 µs | +1.9% | CPP |
| fmi3FreeInstance | 342.00 µs | 348.00 µs | +1.8% | CPP |
| fmi3GetFloat64_1 | 132.00 µs | 135.00 µs | +2.3% | CPP |
| fmi3GetFloat64_10 | 142.00 µs | 145.00 µs | +2.1% | CPP |
| fmi3GetFloat64_100 | 287.00 µs | 293.00 µs | +2.1% | CPP |
| fmi3GetVersion | 118.00 µs | 121.00 µs | +2.5% | CPP |
| fmi3InstantiateCoSimulation | 1450.00 µs | 1478.00 µs | +1.9% | CPP |
| fmi3SetDebugLogging | 125.00 µs | 128.00 µs | +2.4% | CPP |

## Memory Benchmarks

| Benchmark | C++ | Rust | Difference | Winner |
|-----------|-----|------|------------|--------|
| server_10_instances | 46.80 MB | 47.20 MB | +0.9% | CPP |
| server_1_instance | 15.30 MB | 15.60 MB | +2.0% | CPP |
| server_idle | 12.10 MB | 12.40 MB | +2.5% | CPP |
| server_startup | 8.20 MB | 8.50 MB | +3.7% | CPP |

## Throughput Benchmarks

| Benchmark | C++ | Rust | Difference | Winner |
|-----------|-----|------|------------|--------|
| DoStep_100ms | 2953.00 ops/sec | 2948.00 ops/sec | -0.2% | RUST |
| DoStep_10ms | 2921.00 ops/sec | 2905.00 ops/sec | -0.5% | RUST |
| DoStep_1ms | 2847.00 ops/sec | 2819.00 ops/sec | -1.0% | RUST |

## Summary Statistics

| Metric Category | Average Difference (Rust vs C++) |
|----------------|-----------------------------------|
| Latency | +2.12% |
| Throughput | -0.57% |
| Memory | +2.24% |
