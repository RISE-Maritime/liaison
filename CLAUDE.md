# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Liaison is an open-source tool for remote sharing and serving of Functional Mock-up Units (FMUs) while preserving intellectual property. It uses a client-server architecture with Zenoh as the communication layer to handle FMI 3.0 function calls over a network.

**Current branch `port-to-rust-v2`**: Active Rust port in progress. The main branch contains the C++ implementation.

## Build Commands

```bash
# Configure and build
cmake -B build -S .
cmake --build build

# Run unit tests (C/CMocka)
cd build/tests/c && ctest --verbose

# Run E2E tests (Python/pytest)
pytest tests/
```

## Architecture

### Client-Server Model

1. **Server (`liaison --serve <fmu> <responder-id>`)**: Loads the original FMU, creates Zenoh session, exposes FMI functions as RPC endpoints at `rpc/{responderId}/{functionName}`

2. **Client (`liaison --make-fmu <fmu> <responder-id>`)**: Creates a "LiaisonFMU" that replaces the original binary with the liaison shared library. The client FMU exports standard FMI3 functions that make Zenoh queries to the server.

### Key Components

- `src/liaison.cpp` - CLI and server logic, uses macros to declare queryables for each FMI function
- `src/fmi3Functions.cpp` - Client-side FMI3 implementation, makes Zenoh RPC calls
- `src/fmi3.proto` - Protocol Buffer definitions for all FMI3 function parameters/returns
- `src/utils.cpp` - FMU archive handling (zip/unzip operations)

### Output Artifacts

- `build/liaison` - CLI executable
- `build/binaries/x86_64-linux/libliaisonfmu.so` (or `x86_64-windows/liaisonfmu.dll`) - Shared library embedded in LiaisonFMUs

## Testing

The test suite is **implementation-agnostic** - tests validate behavior through CLI and FMI3 C ABI interfaces, working with any Liaison implementation (C++, Rust, Go).

### Unit Tests (tests/c/)

Use CMocka framework with a DummyFMU mock that records all FMI calls to JSON for verification. Test harness manages server lifecycle and library loading.

Run with environment variables to test alternative implementations:
```bash
export LIAISON_BINARY=/path/to/liaison
export LIAISON_FMU_LIB=/path/to/libliaisonfmu.so
export DUMMY_FMU_LIB=/workspace/build/test_fmu/binaries/x86_64-linux/DummyFMU.so
export DUMMY_FMU_PATH=/workspace/build/test_fmu/DummyFMU.fmu
cd build && ctest --verbose
```

### E2E Tests (tests/*.py)

Test complete workflows using FMPy to simulate FMUs. Point to alternative implementations with:
```bash
LIAISON_BIN=/path/to/liaison pytest tests/
```

## FMI 3.0 Implementation Status

**Implemented**: Common functions (Instantiate, Initialize, Terminate, Reset, FreeInstance), all Get/Set operations for Float32/64, Int8/16/32/64, UInt8/16/32/64, Boolean, String, Binary, Clock, and fmi3DoStep.

**Not implemented**: Variable dependencies, FMU state serialization, directional/adjoint derivatives, configuration mode, clock intervals/shifts, discrete states, Model Exchange specific functions.

## Development Environment

Use VSCode with DevContainers extension. The `.devcontainer/` directory contains the complete development environment setup with all dependencies.

## Key Constraints

- FMI 3.0 compliance is mandatory
- Output FMU naming: `<ModelName>Liaison.fmu`
- Binary directory structure: `binaries/x86_64-{linux|windows}/`
- Instance tracking via maps (server maintains `std::unordered_map<int, fmi3Instance>`)
- Platform-specific dynamic library loading (dlopen/dlsym on Linux, LoadLibrary/GetProcAddress on Windows)
