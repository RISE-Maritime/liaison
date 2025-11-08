# Implementation Plan: Port Liaison to Rust

## Overview
This document tracks the progress of porting the Liaison FMI library from C++ to Rust.

## Current C++ Components
1. **liaison.cpp** - Main server executable (~1435 lines)
   - FMU loading and serving
   - Zenoh session management
   - FMU creation utilities
   - Python environment handling

2. **fmi3Functions.cpp** - Client FMU library (~962 lines)
   - FMI 3.0 function implementations
   - Zenoh client communication
   - Placeholder instance management

3. **utils.cpp** - Utilities (~113 lines)
   - Temporary directory creation
   - FMU unzipping
   - File manipulation for ZIP archives

4. **fmi3.proto** - Protocol buffers definition (~409 lines)
   - Message definitions for FMI function calls

## Implementation Steps

### Phase 1: Project Setup
- [x] 1.1 Create Rust project structure (Cargo.toml) - COMPLETED
- [x] 1.2 Add dependencies (zenoh, prost, zip, serde_json, etc.) - COMPLETED
- [x] 1.3 Set up build configuration for cross-platform support - COMPLETED
- [x] 1.4 Configure protobuf code generation with prost-build - COMPLETED

### Phase 2: Core Utilities
- [x] 2.1 Port utils.cpp - temp directory and file utilities - COMPLETED
- [x] 2.2 Port FMU unzipping functionality - COMPLETED
- [x] 2.3 Port ZIP archive manipulation - COMPLETED

### Phase 3: Protocol Buffers
- [x] 3.1 Generate Rust protobuf code from fmi3.proto - COMPLETED
- [x] 3.2 Create type conversion utilities (proto Status <-> FMI status) - COMPLETED

### Phase 4: Client Library (fmi3Functions)
- [ ] 4.1 Port Placeholder class and session management
- [ ] 4.2 Port FMI common functions (GetVersion, SetDebugLogging, etc.)
- [ ] 4.3 Port FMI instantiation functions
- [ ] 4.4 Port FMI get/set value functions (Float32/64, Int*, UInt*, Boolean, String, Binary, Clock)
- [ ] 4.5 Port FMI lifecycle functions (Initialize, Terminate, Reset, etc.)
- [ ] 4.6 Port FMI co-simulation functions (DoStep)
- [ ] 4.7 Set up C ABI export for shared library

### Phase 5: Server Application (liaison)
- [ ] 5.1 Port FMU library loading (platform-specific dlopen/LoadLibrary)
- [ ] 5.2 Port server callback functions
- [ ] 5.3 Port queryable declarations and handlers
- [ ] 5.4 Port FMU serving functionality
- [ ] 5.5 Port FMU creation (--make-fmu) functionality
- [ ] 5.6 Port Python environment handling
- [ ] 5.7 Port command-line argument parsing
- [ ] 5.8 Port main function and error handling

### Phase 6: Testing & Validation
- [ ] 6.1 Create integration tests
- [ ] 6.2 Test with reference FMUs
- [ ] 6.3 Cross-platform testing (Linux/Windows)
- [ ] 6.4 Performance comparison with C++ version

### Phase 7: Build System & Documentation
- [ ] 7.1 Set up GitHub Actions for CI/CD
- [ ] 7.2 Update README with Rust build instructions
- [ ] 7.3 Create migration guide

## Current Status
**Phase:** 3 - Protocol Buffers (COMPLETED)
**Last Updated:** 2025-11-08
**Next Step:** 4.1 - Port Placeholder class and session management

## Completed Work

### Phase 1 Achievements
- Created Rust workspace with two crates:
  - `liaison-fmi`: Client library (cdylib) implementing FMI 3.0 functions
  - `liaison-server`: Server binary for serving FMUs over Zenoh
- Set up all required dependencies:
  - Zenoh 1.0 for communication
  - Prost 0.13 for Protocol Buffers
  - Supporting libraries: serde_json, tracing, anyhow, thiserror, zip, tempfile, clap
- Configured prost-build for automatic protobuf code generation
- Created stub implementations for:
  - FMI 3.0 type definitions and function signatures
  - Utility functions for FMU file operations
  - Server command-line interface
  - Placeholder structure for instance management
- Successfully built both crates with `cargo build`
- Passed `cargo clippy` linting with only expected warnings (unused code for stubs)
- All code compiles cleanly on Linux (x86_64-unknown-linux-gnu)

### Phase 2 Achievements
- Ported all utility functions from utils.cpp to Rust:
  - `create_directories`: Creates directories recursively using std::fs::create_dir_all
  - `create_temp_directory`: Creates temporary directories using tempfile crate
  - `unzip_fmu`: Unzips FMU files using zip crate with full error handling
  - `add_file_to_fmu`: Adds files to ZIP archives with proper buffering
- Fixed deprecation warning by using std::mem::forget instead of deprecated into_path()
- All utility functions use idiomatic Rust patterns (Result types, Path generics)

### Phase 3 Achievements
- Generated Rust protobuf code from fmi3.proto using prost-build
- Created comprehensive type conversion utilities in liaison-fmi/src/conversions.rs:
  - ProtoStatus <-> fmi3Status bidirectional conversions using From traits
  - i32 -> fmi3Status conversion for deserializing status codes
  - Added comprehensive unit tests for all conversion functions
- All tests pass successfully (3 conversion tests)
- Protobuf messages are automatically generated at build time

## Notes
- Maintain C ABI compatibility for the client library (cdylib)
- Use similar error handling patterns as C++ (Result types)
- Keep protobuf definitions unchanged for compatibility
- Target: x86_64-linux and x86_64-windows platforms
