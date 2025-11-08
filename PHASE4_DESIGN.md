# Phase 4 Design: Client Library Implementation

This document contains the design and implementations prepared by subagents for Phase 4 of the Rust port.

## Overview

Phase 4 involves porting the FMI 3.0 client library (`fmi3Functions.cpp`) from C++ to Rust. The subagents have prepared comprehensive implementations for all major components.

## Components Completed by Subagents

### 1. Placeholder Class (`liaison-fmi/src/placeholder.rs`)

**Status**: Design complete, ready for integration

**Key Features**:
- Zenoh session management with Arc for thread-safe sharing
- Configuration loading from `config.json` with TLS certificate path resolution
- Log message subscriber setup for receiving FMU server logs
- Generic `query()` method for Zenoh RPC calls
- Proper cleanup in Drop implementation

**Dependencies**:
- `zenoh` 1.0 for communication
- `prost` for protobuf serialization
- `serde_json` for config parsing
- Platform-specific code for getting library directory

### 2. Utility Functions (`liaison-fmi/src/utils.rs`)

**Status**: Implementation complete

**Key Features**:
- `get_base_directory()` - Platform-specific function to get FMU base directory
- Windows implementation using GetModuleHandleEx/GetModuleFileName
- Unix implementation using dladdr
- Returns grandparent directory of the shared library

**Dependencies Added**:
- `libc = "0.2"` for Unix (dladdr)
- `windows` crate features already configured

### 3. FMI Common Functions (`liaison-fmi/src/fmi3.rs`)

**Status**: Design complete

**Functions Implemented**:
- `fmi3GetVersion()` - Returns "3.0"
- `fmi3SetDebugLogging()` - Configures debug logging via Zenoh
- `fmi3FreeInstance()` - Frees Placeholder instance

**Helper Utilities**:
- `get_placeholder!` macro for safe instance casting
- `zenoh_query()` helper function template
- Protobuf serialization/deserialization

### 4. FMI Instantiation Functions (`liaison-fmi/src/fmi3.rs`)

**Status**: Design complete

**Functions Implemented**:
- `fmi3InstantiateCoSimulation()`
- `fmi3InstantiateModelExchange()`
- `fmi3InstantiateScheduledExecution()`

**Implementation Pattern**:
1. Create Placeholder instance with Zenoh session
2. Build protobuf input message
3. Send Zenoh query to server
4. Parse instance index from response
5. Return Placeholder as fmi3Instance pointer

**Helper Functions**:
- `c_str_to_string()` - Safe C string conversion
- `zenoh_query_instance()` - Zenoh query with error logging
- `log_error()` - FMI callback logging

### 5. FMI Lifecycle Functions (`liaison-fmi/src/fmi3.rs`)

**Status**: Design complete

**Functions Implemented**:
- `fmi3EnterInitializationMode()`
- `fmi3ExitInitializationMode()`
- `fmi3Terminate()`
- `fmi3Reset()`
- `fmi3EnterConfigurationMode()`
- `fmi3ExitConfigurationMode()`

**Implementation Pattern**:
- Cast instance to Placeholder
- Create appropriate protobuf message
- Serialize and send via Zenoh
- Return status from response

### 6. FMI Get/Set Value Functions (`liaison-fmi/src/fmi3.rs`)

**Status**: Design complete with macro-based generation

**Functions Implemented** (for all types):
- Float32/Float64
- Int8/UInt8, Int16/UInt16, Int32/UInt32, Int64/UInt64
- Boolean
- String (with special C string handling)
- Binary (with size arrays)
- Clock

**Macros Defined**:
- `define_fmi3_set_value_function!` - Generates setter functions
- `define_fmi3_get_value_function!` - Generates getter functions

**Helper Traits**:
- `Fmi3InputMessage` - For get message construction
- `Fmi3SetInputMessage<T>` - For set message construction with values

### 7. FMI Co-Simulation Function (`liaison-fmi/src/fmi3.rs`)

**Status**: Design complete

**Function Implemented**:
- `fmi3DoStep()` - Main simulation stepping function

**Features**:
- Handles all input/output parameters
- Reads output parameters before sending
- Uses Placeholder query method for Zenoh communication

## Integration Checklist

To integrate these implementations:

- [ ] 1. Add Zenoh session to Placeholder struct
- [ ] 2. Implement complete Placeholder::new() with config loading
- [ ] 3. Implement Placeholder::query() method
- [ ] 4. Add log message subscriber to Placeholder
- [ ] 5. Integrate utils.rs for get_base_directory()
- [ ] 6. Add all FMI function implementations to fmi3.rs
- [ ] 7. Update lib.rs to include utils module
- [ ] 8. Add necessary imports (prost::Message, std::ffi, etc.)
- [ ] 9. Add zenoh_config crate if needed for JSON5 parsing
- [ ] 10. Update Cargo.toml dependencies if needed
- [ ] 11. Handle protobuf message type naming (verify generated names)
- [ ] 12. Test compilation and fix any errors
- [ ] 13. Add integration tests

## Implementation Notes

### Protobuf Message Naming

The auto-generated protobuf messages follow this naming convention:
- Input messages: `Fmi3<Function>InputMessage`
- Output messages: `Fmi3<Function>OutputMessage` or `Fmi3StatusMessage`
- Example: `Fmi3SetFloat64InputMessage`, `Fmi3GetFloat64OutputMessage`

### Error Handling

All functions use the FMI status codes:
- `fmi3OK` (0) - Success
- `fmi3Warning` (1) - Warning
- `fmi3Discard` (2) - Discard
- `fmi3Error` (3) - Error
- `fmi3Fatal` (4) - Fatal error

Conversion between protobuf Status and fmi3Status is handled by the conversions module.

### Memory Management

- Placeholder instances are heap-allocated using Box
- Converted to raw pointers for C ABI (`Box::into_raw`)
- Freed in `fmi3FreeInstance` using `Box::from_raw`
- String returns in `fmi3GetString` use `CString::into_raw` (caller responsible for freeing)

### Thread Safety

- Placeholder is Send but not Sync (single-threaded FMI usage)
- Zenoh session uses Arc for potential multi-threading
- No additional synchronization needed for basic FMI usage

## Testing Strategy

1. **Unit Tests**: Test individual functions with mock Zenoh responses
2. **Integration Tests**: Test with actual FMU server
3. **Cross-platform Tests**: Verify on Linux and Windows
4. **Memory Tests**: Verify no leaks using valgrind/Address Sanitizer
5. **Compatibility Tests**: Test with existing C++ FMU servers

## Known Limitations

1. Zenoh query implementation is incomplete (marked with TODO)
2. String memory management in GetString needs careful handling
3. Binary data handling assumes contiguous buffer layout
4. Configuration file path resolution needs testing on Windows
5. TLS certificate path handling needs validation

## Next Steps

1. Integrate Placeholder implementation
2. Complete Zenoh query method
3. Test compilation
4. Add integration tests
5. Benchmark performance vs C++ version
6. Document API differences
