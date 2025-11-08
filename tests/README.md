# Liaison Integration Tests

This directory contains end-to-end integration tests for the Liaison FMI client-server system.

## Overview

The test framework verifies the full client-server interaction by:

1. Starting a liaison-server instance with a test FMU
2. Using Zenoh to communicate with the server
3. Calling FMI 3.0 functions through the protocol
4. Verifying that responses match expected behavior

## Structure

```
tests/
├── README.md                    # This file
├── integration_test.rs          # Main integration tests
├── utils/                       # Test utilities
│   └── mod.rs                  # Server lifecycle, helpers
└── fixtures/                    # Test FMU fixtures
    ├── mock_fmu.c              # Mock FMU implementation
    ├── modelDescription.xml     # FMU model description
    ├── build_mock_fmu.sh       # Build script
    └── build/                   # Build output (generated)
        └── MockFMU.fmu         # Compiled test FMU
```

## Test Categories

### 1. Server Lifecycle Tests
- `test_server_starts`: Verifies server can start successfully
- Server automatically stops when test completes

### 2. Basic Function Tests
- `test_get_version`: Tests FMI version query
- Validates protocol communication

### 3. Instance Management Tests
- `test_instantiate_cosimulation`: Tests FMU instantiation
- `test_multiple_instances`: Verifies multiple instances can coexist

### 4. Simulation Tests
- `test_full_simulation_cycle`: Complete simulation workflow
  - Instantiate
  - Enter/exit initialization mode
  - Enter step mode
  - Perform multiple steps
  - Get variable values
  - Terminate
  - Free instance

### 5. Variable Access Tests
- `test_set_and_get_variables`: Tests variable read/write operations
- Validates data correctness

## Mock FMU

The test framework includes a simple mock FMU (`MockFMU`) with the following characteristics:

### Variables

| Name      | Value Ref | Causality | Type    | Description          |
|-----------|-----------|-----------|---------|----------------------|
| time      | 0         | output    | Float64 | Simulation time      |
| counter   | 1         | output    | Float64 | Step counter         |
| stepSize  | 2         | parameter | Float64 | Simulation step size |

### Behavior

- **Counter**: Increments by 1.0 on each `doStep`
- **Time**: Tracks simulation time
- **Thread-safe**: Supports multiple instances

### Building the Mock FMU

The mock FMU is automatically built when running tests. To build manually:

```bash
cd tests/fixtures
./build_mock_fmu.sh
```

This creates `tests/fixtures/build/MockFMU.fmu`.

## Running Tests

### Prerequisites

1. **Build the liaison server:**
   ```bash
   cargo build --bin liaison
   ```

2. **Install dependencies:**
   - GCC or Clang (for building mock FMU)
   - zip/unzip utilities
   - Zenoh runtime (included with Rust dependencies)

### Run All Integration Tests

```bash
cargo test --test integration_test
```

### Run Specific Tests

```bash
# Run a single test
cargo test --test integration_test test_full_simulation_cycle

# Run with output
cargo test --test integration_test -- --nocapture

# Run with debug logging
RUST_LOG=debug cargo test --test integration_test -- --nocapture
```

### Parallel Execution

Tests use unique responder IDs to avoid conflicts, allowing parallel execution:

```bash
cargo test --test integration_test -- --test-threads=4
```

## Test Utilities

### ServerHandle

Manages liaison server lifecycle:

```rust
use utils::{ServerHandle, ServerConfig};

let server = ServerHandle::start(ServerConfig {
    fmu_path: PathBuf::from("path/to/fmu"),
    responder_id: "test_server".to_string(),
    zenoh_config: None,
    debug: true,
})?;

// Server automatically stops when dropped
```

### TestFixture

Complete test environment with server and Zenoh session:

```rust
let fixture = TestFixture::new().await?;

// Query the server
let response: StatusMessage = fixture
    .query("fmi3GetVersion", &payload)
    .await?;
```

### Utility Functions

- `ensure_mock_fmu_built()`: Builds mock FMU if needed
- `get_workspace_dir()`: Finds workspace root
- `get_fixtures_dir()`: Returns fixtures directory
- `wait_for(condition, timeout)`: Waits for condition
- `create_test_zenoh_session()`: Creates Zenoh session

## Writing New Tests

### Basic Test Structure

```rust
#[tokio::test]
async fn test_my_feature() -> Result<()> {
    // Create test fixture (starts server)
    let fixture = TestFixture::new().await?;

    // Prepare message
    let msg = MyMessage { /* ... */ };
    let mut payload = Vec::new();
    msg.encode(&mut payload)?;

    // Query server
    let response: MyResponse = fixture
        .query("fmi3MyFunction", &payload)
        .await?;

    // Verify response
    assert_eq!(response.status, Status::Ok as i32);

    Ok(())
}
```

### Best Practices

1. **Use TestFixture**: Handles server lifecycle automatically
2. **Unique Names**: Use unique instance names to avoid conflicts
3. **Cleanup**: Servers auto-cleanup via Drop trait
4. **Timeouts**: Use reasonable timeouts (5-10 seconds)
5. **Error Context**: Use `.context()` for better error messages
6. **Logging**: Use `println!` for test progress (with `--nocapture`)

## Troubleshooting

### Server Won't Start

- **Check binary**: Ensure `cargo build` completed successfully
- **Check ports**: Zenoh may have port conflicts
- **Check logs**: Run with `RUST_LOG=debug`

```bash
RUST_LOG=debug cargo test --test integration_test -- --nocapture
```

### Mock FMU Build Fails

- **Install gcc/clang**: Required for C compilation
- **Check script**: Verify `build_mock_fmu.sh` is executable
- **Manual build**: Try building manually to see errors

```bash
cd tests/fixtures
bash -x ./build_mock_fmu.sh
```

### Tests Timeout

- **Increase timeout**: Modify `Duration::from_secs(5)` in test
- **Check server**: Verify server process is running
- **Zenoh issues**: Check Zenoh configuration

### Tests Hang

- **Server cleanup**: Ensure servers are stopped (check `ps aux | grep liaison`)
- **Kill manually**: `pkill liaison`
- **Restart Zenoh**: Sometimes Zenoh needs restart

## Integration with CI/CD

### GitHub Actions Example

```yaml
name: Integration Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install dependencies
        run: sudo apt-get install -y gcc zip

      - name: Build
        run: cargo build --all

      - name: Run integration tests
        run: cargo test --test integration_test
        env:
          RUST_LOG: info
```

## Performance Considerations

- **Test duration**: Full suite runs in ~30 seconds
- **Server startup**: ~2 seconds per server
- **Mock FMU build**: ~1 second (cached after first build)
- **Parallel tests**: Safe with unique responder IDs

## Future Enhancements

- [ ] Add performance benchmarks
- [ ] Test error scenarios (invalid messages, timeouts)
- [ ] Test Zenoh configuration options
- [ ] Add Model Exchange tests
- [ ] Add Scheduled Execution tests
- [ ] Test Python FMU support
- [ ] Add stress tests (many instances, large messages)
- [ ] Test network partition scenarios
- [ ] Add test coverage reporting

## Contributing

When adding new tests:

1. Follow existing patterns (use TestFixture)
2. Document test purpose clearly
3. Add test to appropriate category
4. Update this README if adding new features
5. Ensure tests pass in CI/CD

## License

Same as parent project (MIT).
