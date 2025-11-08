# Integration Tests Quick Start

Get up and running with the Liaison integration tests in 5 minutes.

## Quick Setup

```bash
# 1. Build the project
cargo build --bin liaison

# 2. Run the tests
cargo test --test integration_test
```

That's it! The test framework will automatically:
- Build the mock FMU
- Start liaison servers
- Run all integration tests
- Clean up resources

## Your First Test Run

Expected output:

```
   Compiling liaison-server v0.1.0
   Compiling liaison-fmi v0.1.0
    Finished test [unoptimized + debuginfo] target(s) in 15.23s
     Running tests/integration_test.rs

running 5 tests
Building mock FMU...
Mock FMU built successfully
test test_server_starts ... ok
test test_get_version ... ok
test test_instantiate_cosimulation ... ok
test test_set_and_get_variables ... ok
test test_full_simulation_cycle ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Run Individual Tests

```bash
# Test server lifecycle
cargo test --test integration_test test_server_starts

# Test complete simulation
cargo test --test integration_test test_full_simulation_cycle

# With debug output
cargo test --test integration_test test_full_simulation_cycle -- --nocapture
```

## Debug Mode

See what's happening under the hood:

```bash
RUST_LOG=debug cargo test --test integration_test -- --nocapture
```

## Common Issues

### "Could not find liaison server binary"
**Solution:** Run `cargo build --bin liaison` first

### "Failed to build mock FMU"
**Solution:** Install gcc/clang: `sudo apt-get install gcc` (Linux) or `xcode-select --install` (macOS)

### Tests hang or timeout
**Solution:** Kill any lingering servers: `pkill liaison`

## What Gets Tested?

1. ✅ Server starts successfully
2. ✅ FMI version query works
3. ✅ FMU instantiation works
4. ✅ Full simulation cycle (init → step → terminate)
5. ✅ Variable get/set operations
6. ✅ Multiple instances simultaneously

## Next Steps

- Read the full [README.md](README.md) for detailed documentation
- Add your own tests to `integration_test.rs`
- Check out the mock FMU implementation in `fixtures/mock_fmu.c`

## Need Help?

Check the detailed [README.md](README.md) for:
- Complete API documentation
- Troubleshooting guide
- How to write custom tests
- CI/CD integration examples
