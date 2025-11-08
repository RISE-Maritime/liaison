# Liaison FMI Client Library - Integration Tests

This directory contains comprehensive integration tests for the liaison-fmi client library.

## Test Files

### `integration_tests.rs`
Comprehensive integration tests covering the full FMI 3.0 client library functionality.

**Test Coverage:**

1. **Conversion Functions** (15 tests)
   - Proto Status to FMI Status conversions (all variants)
   - FMI Status to Proto Status conversions (all variants)
   - i32 to FMI Status conversions (valid and invalid values)
   - Round-trip conversion consistency
   - Status equality and comparison

2. **FMI Function Exports** (13 tests)
   - `fmi3GetVersion` - Version string verification
   - `fmi3SetDebugLogging` - Debug logging configuration
   - `fmi3FreeInstance` - Instance cleanup
   - `fmi3Terminate` - Simulation termination
   - `fmi3Reset` - Instance reset
   - `fmi3EnterInitializationMode` - Initialization entry
   - `fmi3ExitInitializationMode` - Initialization exit
   - `fmi3EnterConfigurationMode` - Configuration entry
   - `fmi3ExitConfigurationMode` - Configuration exit
   - `fmi3DoStep` - Co-simulation step
   - `fmi3GetBoolean` / `fmi3SetBoolean` - Boolean variable access
   - `fmi3GetString` / `fmi3SetString` - String variable access

3. **Instantiation Functions** (3 tests)
   - `fmi3InstantiateCoSimulation` - Co-Simulation instantiation
   - `fmi3InstantiateModelExchange` - Model Exchange instantiation
   - `fmi3InstantiateScheduledExecution` - Scheduled Execution instantiation

4. **Error Handling** (3 tests)
   - Null instance pointer safety (all functions)
   - Null parameter handling
   - Safe failure modes without crashes

5. **Type Safety** (6 tests)
   - FMI type sizes (Float32, Float64, Int8-64, UInt8-64, Boolean)
   - FMI pointer type sizes (Instance, String, Binary, etc.)
   - FMI Status enum values and equality
   - Value reference types and arrays

6. **Callback Functions** (3 tests)
   - Callback type definitions (Option types)
   - Log message callback function pointers
   - Callback invocation safety

7. **String Handling** (2 tests)
   - C string creation and conversion
   - Null FMI string handling

8. **Proto Integration** (2 tests)
   - Proto status enum values
   - Proto status from i32 conversion

### `conversion_tests.rs`
Focused unit tests for type conversion functionality.

**Test Coverage:**

1. **Exhaustive Conversions** (2 tests)
   - All proto::Status to fmi3Status conversions
   - All fmi3Status to proto::Status conversions

2. **i32 Conversions** (2 tests)
   - Valid i32 values to fmi3Status
   - Invalid i32 values (should default to Error)

3. **Bidirectional Consistency** (1 test)
   - Round-trip proto → fmi → proto preserves values

4. **Numeric Representation** (2 tests)
   - FMI status numeric values
   - Proto status numeric values

5. **Pattern Matching** (1 test)
   - Conversions work correctly in match expressions

## Running the Tests

```bash
# Run all liaison-fmi tests
cargo test -p liaison-fmi

# Run only integration tests
cargo test -p liaison-fmi --test integration_tests

# Run only conversion tests
cargo test -p liaison-fmi --test conversion_tests

# Run a specific test
cargo test -p liaison-fmi test_proto_status_to_fmi_status_conversions

# Run tests with output
cargo test -p liaison-fmi -- --nocapture
```

## Test Design Philosophy

### Mock-Based Testing
These tests are designed to verify the client library's behavior **without** requiring a running Zenoh server or FMU. They focus on:

- **Type safety**: Ensuring correct type conversions and sizes
- **API surface**: Verifying all FMI functions are exported
- **Error handling**: Testing null pointer safety and graceful failures
- **Interface contracts**: Validating function signatures and return values

### What Is NOT Tested
The following require a full Zenoh server and FMU instance:

- Placeholder initialization with real config.json
- Actual Zenoh query/response communication
- Log message subscriber functionality
- Full instantiation with valid parameters
- Real variable getter/setter operations with server

For these scenarios, end-to-end integration tests should be run with a live Zenoh server.

## Coverage Summary

| Category | Tests | Status |
|----------|-------|--------|
| Conversions | 15 | ✓ Complete |
| FMI Exports | 13 | ✓ Complete |
| Instantiation | 3 | ✓ Complete |
| Error Handling | 3 | ✓ Complete |
| Type Safety | 6 | ✓ Complete |
| Callbacks | 3 | ✓ Complete |
| String Handling | 2 | ✓ Complete |
| Proto Integration | 2 | ✓ Complete |
| **Total** | **47+** | **✓ Complete** |

## Adding New Tests

When adding new functionality to liaison-fmi, please add corresponding tests:

1. **Unit tests** in the module's `#[cfg(test)]` section for internal logic
2. **Integration tests** in this directory for public API verification
3. **Conversion tests** if adding new type conversions

### Test Template

```rust
#[test]
fn test_your_feature() {
    // Arrange: Set up test data
    let test_value = some_value();

    // Act: Execute the functionality
    let result = your_function(test_value);

    // Assert: Verify the expected behavior
    assert_eq!(result, expected_value);
}
```

## Continuous Integration

These tests are designed to run in CI environments without external dependencies:

- No Zenoh server required
- No file system dependencies (except for type checking)
- No network access needed
- Fast execution (< 1 second for full suite)

## Known Limitations

1. **Config file testing**: Tests that require config.json loading cannot run without a mock file system
2. **Zenoh communication**: Cannot test actual query/response without a server
3. **Thread safety**: Some thread safety properties are compile-time only
4. **Platform-specific**: Some tests may behave differently on Windows vs Unix

## Future Enhancements

- [ ] Add property-based testing with `proptest` or `quickcheck`
- [ ] Mock Zenoh session for more comprehensive testing
- [ ] Add performance benchmarks
- [ ] Add fuzzing tests for string handling
- [ ] Create test fixtures for common scenarios
