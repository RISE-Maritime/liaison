# Test Framework Cheat Sheet

Quick reference for common tasks.

## Run Tests

```bash
# All tests
cargo test --test integration_test

# Specific test
cargo test --test integration_test test_full_simulation_cycle

# With output
cargo test --test integration_test -- --nocapture

# With debug logging
RUST_LOG=debug cargo test --test integration_test -- --nocapture

# Parallel execution
cargo test --test integration_test -- --test-threads=4
```

## Make Commands

```bash
cd tests
make test              # Run all tests
make test-verbose      # Run with stdout
make test-debug        # Run with debug logs
make test-one TEST=name  # Run one test
make build             # Build server only
make clean             # Clean artifacts
```

## Write a Test

```rust
#[tokio::test]
async fn test_my_feature() -> Result<()> {
    let fixture = TestFixture::new().await?;

    let msg = MyMessage { /* fields */ };
    let mut payload = Vec::new();
    msg.encode(&mut payload)?;

    let response: MyResponse = fixture
        .query("fmi3Function", &payload)
        .await?;

    assert_eq!(response.status, Status::Ok as i32);
    Ok(())
}
```

## Mock FMU Variables

| Name     | VR | Type    | Description |
|----------|----|---------| ------------|
| time     | 0  | Float64 | Sim time    |
| counter  | 1  | Float64 | Step count  |
| stepSize | 2  | Float64 | Step size   |

## Query Pattern

```rust
let msg = /* protobuf message */;
let mut payload = Vec::new();
msg.encode(&mut payload)?;

let response: ResponseType = fixture
    .query("fmi3FunctionName", &payload)
    .await?;
```

## Debugging

```bash
# View logs
RUST_LOG=liaison_server=debug cargo test --test integration_test -- --nocapture

# Kill hung servers
pkill liaison

# Manual FMU build
cd tests/fixtures && ./build_mock_fmu.sh

# Check server binary
ls -lh target/debug/liaison
```

## File Locations

- Tests: `/workspace/tests/integration_test.rs`
- Utils: `/workspace/tests/utils/mod.rs`
- Mock FMU: `/workspace/tests/fixtures/mock_fmu.c`
- Docs: `/workspace/tests/README.md`

## Common Issues

| Problem | Solution |
|---------|----------|
| "Binary not found" | `cargo build --bin liaison` |
| "FMU build failed" | Install gcc/clang |
| Tests hang | `pkill liaison` |
| Port conflicts | Tests use unique IDs |

## Documentation

- `README.md` - Full documentation
- `QUICKSTART.md` - 5-min start
- `EXTENDING.md` - Write tests
- `CHEATSHEET.md` - This file

## Quick Workflow

```bash
# 1. Build
cargo build --bin liaison

# 2. Run tests
cargo test --test integration_test

# 3. Debug a test
RUST_LOG=debug cargo test --test integration_test test_name -- --nocapture
```

## Test Structure

```
TestFixture::new()
  → ensure_mock_fmu_built()
  → ServerHandle::start()
  → create_test_zenoh_session()

fixture.query(func, payload)
  → encode message
  → send Zenoh query
  → receive reply
  → decode response

Drop TestFixture
  → ServerHandle::drop()
  → graceful shutdown
```

## Pro Tips

- Use `-- --nocapture` to see println output
- Use `RUST_LOG=debug` for detailed logs
- Tests are parallel-safe (unique IDs)
- Servers auto-cleanup on test end
- FMU builds automatically once
