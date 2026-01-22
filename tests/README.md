# Liaison Test Suite

This document explains how the tests are structured and how they can be used to validate different implementations of Liaison (e.g., C++, Rust, Go).

## Test Architecture Overview

The test suite is **implementation-agnostic** by design. Tests interact with Liaison through two stable interfaces:

1. **CLI interface** (`liaison --make-fmu`, `liaison --serve`) - used by E2E tests
2. **FMI3 C ABI** (standard function signatures like `fmi3GetFloat64`) - used by unit tests

This means **the same tests can validate any implementation** that conforms to these interfaces, regardless of the programming language used.

## Test Types

### End-to-End Tests (Python/pytest)

Located in `/tests/*.py`

These tests validate the complete system by:
- Invoking the `liaison` CLI binary
- Creating LiaisonFMUs and running simulations with FMPy
- Verifying physics results and FMU structure

**Files:**
- `test_e2e_bouncing_ball.py` - Tests with native C FMU
- `test_e2e_bouncing_ball_python.py` - Tests with Python-based FMU
- `conftest.py` - Pytest fixtures and configuration

### Unit Tests (C/CMocka)

Located in `/tests/c/`

These tests validate the FMI3 protocol by:
- Loading the LiaisonFMU shared library via `dlopen`
- Calling FMI3 functions through C function pointers
- Verifying that calls are correctly transmitted to the server

**Test files in `/tests/c/tests/`:**
- `test_instantiation.c` - FMU instantiation modes
- `test_initialization.c` - Initialization workflow
- `test_float64_values.c` - Float64 get/set operations
- `test_integer_values.c` - Integer type operations
- `test_boolean_values.c` - Boolean operations
- `test_string_values.c` - String handling
- `test_binary_values.c` - Binary data handling
- `test_clock_values.c` - Clock operations
- `test_do_step.c` - Simulation stepping
- `test_state_management.c` - State transitions

---

## Using Tests with Different Implementations

### E2E Tests

The E2E tests only need to know the path to the `liaison` binary. They don't care what language it's written in.

#### Option 1: Modify `conftest.py`

Edit the paths at the top of `conftest.py`:

```python
# Current (C++ implementation)
BUILD_DIR = PROJECT_ROOT / "build"
LIAISON_BIN = BUILD_DIR / "liaison"

# For Rust implementation
BUILD_DIR = PROJECT_ROOT / "target/release"
LIAISON_BIN = BUILD_DIR / "liaison"

# For Go implementation
BUILD_DIR = PROJECT_ROOT / "bin"
LIAISON_BIN = BUILD_DIR / "liaison"
```

#### Option 2: Use environment variables (recommended)

Modify `conftest.py` to read from environment variables:

```python
import os

BUILD_DIR = Path(os.environ.get("LIAISON_BUILD_DIR", PROJECT_ROOT / "build"))
LIAISON_BIN = Path(os.environ.get("LIAISON_BIN", BUILD_DIR / "liaison"))
```

Then run tests with:

```bash
# Rust implementation
LIAISON_BUILD_DIR=/path/to/rust/target/release \
LIAISON_BIN=/path/to/rust/target/release/liaison \
pytest tests/

# Go implementation
LIAISON_BIN=/path/to/go/bin/liaison pytest tests/
```

#### Requirements for the implementation

The E2E tests expect:
1. A `liaison` executable that accepts `--make-fmu <fmu> <responder-id>` and `--serve <fmu> <responder-id>`
2. A `binaries/` directory containing the LiaisonFMU shared library
3. Output FMU files to follow the naming convention `<Name>Liaison.fmu`

---

### Unit Tests (C)

The C unit tests use dynamic library loading (`dlopen`/`dlsym`), making them **language-agnostic**. They will work with any shared library that exports the FMI3 C ABI.

#### Environment Variables

The test harness reads paths from these environment variables:

| Variable | Description | Example |
|----------|-------------|---------|
| `LIAISON_BINARY` | Path to the liaison executable | `/workspace/build/liaison` |
| `LIAISON_FMU_LIB` | Path to the LiaisonFMU shared library | `/workspace/build/binaries/x86_64-linux/libliaisonfmu.so` |
| `DUMMY_FMU_LIB` | Path to the test mock FMU library | `/workspace/build/test_fmu/binaries/x86_64-linux/DummyFMU.so` |
| `DUMMY_FMU_PATH` | Path to the DummyFMU.fmu archive | `/workspace/build/test_fmu/DummyFMU.fmu` |

#### Running with a Rust Implementation

```bash
# Build your Rust implementation
cd /path/to/rust-liaison
cargo build --release

# Set environment variables pointing to Rust artifacts
export LIAISON_BINARY=/path/to/rust-liaison/target/release/liaison
export LIAISON_FMU_LIB=/path/to/rust-liaison/target/release/libliaisonfmu.so

# Keep using the C test infrastructure (DummyFMU)
export DUMMY_FMU_LIB=/workspace/build/test_fmu/binaries/x86_64-linux/DummyFMU.so
export DUMMY_FMU_PATH=/workspace/build/test_fmu/DummyFMU.fmu

# Run the tests
cd /workspace/build
ctest --verbose
```

#### Running with a Go Implementation

```bash
# Build your Go implementation (must produce C-compatible shared library)
cd /path/to/go-liaison
go build -buildmode=c-shared -o libliaisonfmu.so ./fmu

# Set environment variables
export LIAISON_BINARY=/path/to/go-liaison/liaison
export LIAISON_FMU_LIB=/path/to/go-liaison/libliaisonfmu.so
export DUMMY_FMU_LIB=/workspace/build/test_fmu/binaries/x86_64-linux/DummyFMU.so
export DUMMY_FMU_PATH=/workspace/build/test_fmu/DummyFMU.fmu

# Run the tests
ctest --verbose
```

#### Requirements for the Implementation

For the unit tests to work, your implementation must:

1. **Export FMI3 functions with C ABI**

   For Rust:
   ```rust
   #[no_mangle]
   pub extern "C" fn fmi3InstantiateCoSimulation(
       instanceName: fmi3String,
       instantiationToken: fmi3String,
       // ... other parameters
   ) -> fmi3Instance {
       // implementation
   }
   ```

   For Go:
   ```go
   //export fmi3InstantiateCoSimulation
   func fmi3InstantiateCoSimulation(
       instanceName *C.char,
       instantiationToken *C.char,
       // ... other parameters
   ) C.fmi3Instance {
       // implementation
   }
   ```

2. **Use standard FMI3 function names** (no name mangling)

3. **Follow the FMI3 calling conventions** (parameter types, return values)

---

## Test Infrastructure Components

### DummyFMU (`/tests/c/dummy_fmu/`)

A mock FMU that records all FMI3 function calls to a JSON file. This allows tests to verify that parameters are correctly transmitted through the Liaison system.

The DummyFMU is **implementation-agnostic** - it's used to test any Liaison implementation by acting as the "server-side" FMU.

### Test Harness (`/tests/c/test_harness/`)

Manages:
- Starting/stopping the `liaison --serve` process
- Loading the LiaisonFMU shared library
- Binding FMI3 function pointers
- Reading and parsing the call log from DummyFMU

### Call Recording

The DummyFMU writes JSON call records to `/tmp/dummy_fmu_calls_<pid>.json`. The test harness reads this file to verify that:
- The correct FMI3 function was called
- Parameters were transmitted accurately
- Return values are correct

---

## Quick Reference

### Running All Tests (Current C++ Implementation)

```bash
# Build
cmake -B build -S .
cmake --build build

# E2E tests
pytest tests/

# Unit tests
cd build && ctest --verbose
```

### Running Tests Against Alternative Implementation

```bash
# E2E tests - just point to your binary
LIAISON_BIN=/path/to/your/liaison pytest tests/

# Unit tests - set all required paths
export LIAISON_BINARY=/path/to/your/liaison
export LIAISON_FMU_LIB=/path/to/your/libliaisonfmu.so
export DUMMY_FMU_LIB=/workspace/build/test_fmu/binaries/x86_64-linux/DummyFMU.so
export DUMMY_FMU_PATH=/workspace/build/test_fmu/DummyFMU.fmu
cd build && ctest --verbose
```

---

## Summary

| Test Type | Interface | Language Agnostic? | Configuration |
|-----------|-----------|-------------------|---------------|
| E2E (pytest) | CLI + FMI standard | Yes | Change `LIAISON_BIN` path |
| Unit (CMocka) | FMI3 C ABI | Yes | Set environment variables |

The tests validate **behavior and protocol compliance**, not implementation details. Any implementation that:
1. Provides a compatible `liaison` CLI
2. Exports FMI3 functions with C ABI

...can be validated using this test suite without modification.
