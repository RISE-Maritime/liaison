# Liaison - Rust Porting Plan

## Executive Summary

This document outlines a comprehensive plan for porting the **Liaison** codebase from C++ to Rust. Liaison is an open-source tool that enables sharing of Functional Mock-up Units (FMUs) over Zenoh networks using a client-server architecture.

**Current Codebase:**
- ~2,527 lines of C++17 code
- ~409 lines of Protobuf definitions
- Actively maintained with recent commits
- Cross-platform (Linux/Windows)

**Key Benefits of Rust Port:**
- **Native Zenoh Integration**: Zenoh itself is written in Rust
- **Memory Safety**: Eliminate entire classes of bugs (use-after-free, data races)
- **Better Error Handling**: Result/Option types for robust error propagation
- **Easier Cross-Compilation**: Cargo's excellent toolchain management
- **Modern Tooling**: Integrated testing, documentation, and dependency management
- **Smaller Binaries**: With optimized builds and selective feature compilation
- **Performance**: Comparable or better performance with zero-cost abstractions

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Rust Crate Ecosystem Mapping](#rust-crate-ecosystem-mapping)
3. [Phased Migration Strategy](#phased-migration-strategy)
4. [Detailed Component Analysis](#detailed-component-analysis)
5. [Critical Challenges & Solutions](#critical-challenges--solutions)
6. [Testing Strategy](#testing-strategy)
7. [Build & Deployment](#build--deployment)
8. [Timeline & Resource Estimates](#timeline--resource-estimates)
9. [Risk Assessment](#risk-assessment)
10. [Success Criteria](#success-criteria)

---

## Architecture Overview

### Current C++ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Simulation Tool (FMPy)                   │
└──────────────────────┬──────────────────────────────────────┘
                       │ FMI 3.0 C API
                       ▼
┌─────────────────────────────────────────────────────────────┐
│              LiaisonFMU (Client Library)                    │
│  - fmi3Functions.cpp: FMI 3.0 function implementations      │
│  - Placeholder class: Zenoh session management              │
│  - Protobuf serialization                                   │
└──────────────────────┬──────────────────────────────────────┘
                       │ Zenoh RPC (Protocol Buffers)
                       ▼
┌─────────────────────────────────────────────────────────────┐
│              Zenoh Network (P2P/Router)                     │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│              Liaison Server (liaison.cpp)                   │
│  - startServer(): Main RPC handler loop                     │
│  - makeFmu(): FMU packaging tool                            │
│  - Dynamic FMU library loading (.so/.dll)                   │
│  - FMI function delegation to original FMU                  │
└─────────────────────────────────────────────────────────────┘
```

### Core Data Flow

1. **Client Request**: Simulation tool calls FMI function → Client library serializes to Protobuf
2. **Network Transport**: Zenoh routes message to server via key expression `rpc/{responder_id}/{function}`
3. **Server Processing**: Server deserializes, calls original FMU function, serializes response
4. **Response**: Client deserializes response and returns to simulation tool

---

## Rust Crate Ecosystem Mapping

### Direct Replacements

| C++ Dependency | Purpose | Rust Crate | Rationale |
|----------------|---------|------------|-----------|
| zenoh-c/cpp | Messaging | `zenoh` (v1.0+) | Native Rust implementation, better API |
| Protocol Buffers 3.20 | Serialization | `prost` + `prost-build` | Idiomatic Rust codegen, smaller binaries |
| libzip 1.10.1 | ZIP handling | `zip` v2.2+ | Pure Rust, no C dependencies |
| spdlog v1.15.1 | Logging | `tracing` + `tracing-subscriber` | Structured logging, better async support |
| nlohmann/json | JSON parsing | `serde_json` | De facto standard, excellent performance |
| zlib | Compression | Built into `zip` crate | No separate dependency needed |

### Additional Rust Crates

| Crate | Purpose | Version | Priority |
|-------|---------|---------|----------|
| `clap` | CLI parsing | v4.5+ | High (replaces manual argv parsing) |
| `anyhow` | Error handling | v1.0+ | High (server application) |
| `thiserror` | Error types | v1.0+ | High (library code) |
| `libloading` | Dynamic library loading | v0.8+ | Critical (FMU loading) |
| `tokio` | Async runtime | v1.40+ | Medium (if async Zenoh APIs used) |
| `tempfile` | Temporary files | v3.13+ | High (FMU extraction) |
| `roxmltree` | XML parsing | v0.20+ | High (modelDescription.xml) |
| `cfg-if` | Platform-specific code | v1.0+ | Medium |
| `once_cell` | Lazy statics | v1.20+ | Medium |

---

## Phased Migration Strategy

### Phase 0: Preparation (1 week)

**Objective**: Set up Rust project structure and validate dependencies

**Tasks**:
1. Create new `liaison-rs` directory (parallel to existing C++ code)
2. Initialize Cargo workspace with three crates:
   ```
   liaison-rs/
   ├── Cargo.toml               # Workspace manifest
   ├── liaison-server/          # Server binary
   │   ├── Cargo.toml
   │   └── src/
   │       └── main.rs
   ├── liaison-fmu/             # Client shared library
   │   ├── Cargo.toml
   │   ├── build.rs             # Protobuf codegen
   │   └── src/
   │       └── lib.rs
   └── liaison-common/          # Shared code (protobuf messages, utils)
       ├── Cargo.toml
       ├── build.rs
       └── src/
           └── lib.rs
   ```
3. Set up CI pipeline for Rust builds (parallel to existing C++ CI)
4. Create feature parity checklist
5. Document build process

**Deliverables**:
- [ ] Working Cargo workspace
- [ ] All dependencies compile successfully on Linux and Windows
- [ ] CI pipeline builds but produces minimal functionality

**Dependencies Validation**:
```toml
# Test compilation of all key dependencies
[dependencies]
zenoh = "1.0"
prost = "0.13"
zip = "2.2"
tracing = "0.1"
serde_json = "1.0"
clap = { version = "4.5", features = ["derive"] }
anyhow = "1.0"
thiserror = "1.0"
libloading = "0.8"
tempfile = "3.13"
roxmltree = "0.20"
```

---

### Phase 1: Core Infrastructure (2-3 weeks)

**Objective**: Port utility code, protobuf definitions, and establish FMI C API boundary

#### 1.1 Protobuf Message Definitions

**File**: `liaison-common/proto/fmi3.proto` (copy from src/fmi3.proto)

**Build Script** (`liaison-common/build.rs`):
```rust
fn main() {
    prost_build::compile_protos(&["proto/fmi3.proto"], &["proto/"])
        .expect("Failed to compile protobuf definitions");
}
```

**Generated Code**: Auto-generated in `OUT_DIR`, imported via:
```rust
// liaison-common/src/lib.rs
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/fmi3.rs"));
}
```

#### 1.2 FMI 3.0 Type Bindings

**File**: `liaison-common/src/fmi3_types.rs`

Port FMI types from `external/fmi3/fmi3PlatformTypes.h`:
```rust
// Core FMI types with C ABI compatibility
#[repr(C)]
pub struct fmi3Instance {
    // Opaque pointer type
    _private: [u8; 0],
}

#[repr(C)]
pub type fmi3Float32 = f32;

#[repr(C)]
pub type fmi3Float64 = f64;

#[repr(C)]
pub type fmi3Int8 = i8;

// ... (all numeric types)

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum fmi3Status {
    OK = 0,
    Warning = 1,
    Discard = 2,
    Error = 3,
    Fatal = 4,
}

// Function pointer types
pub type fmi3LogMessageCallback = unsafe extern "C" fn(
    instance_environment: *mut std::ffi::c_void,
    status: fmi3Status,
    category: *const std::os::raw::c_char,
    message: *const std::os::raw::c_char,
);
```

#### 1.3 Utility Functions

**File**: `liaison-common/src/utils.rs`

Port utilities from `src/utils.cpp`:

```rust
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use tempfile::TempDir;
use zip::ZipArchive;
use std::fs::File;

/// Extract FMU (ZIP archive) to temporary directory
pub fn extract_fmu(fmu_path: &Path) -> Result<TempDir> {
    let file = File::open(fmu_path)
        .context("Failed to open FMU file")?;

    let mut archive = ZipArchive::new(file)
        .context("Failed to read FMU as ZIP archive")?;

    let temp_dir = tempfile::Builder::new()
        .prefix("liaison.")
        .tempdir()
        .context("Failed to create temporary directory")?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = temp_dir.path().join(file.name());

        if file.is_dir() {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                std::fs::create_dir_all(p)?;
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(temp_dir)
}

/// Get platform-specific binary directory name
pub fn get_platform_dir() -> &'static str {
    if cfg!(target_os = "linux") {
        "binaries/x86_64-linux"
    } else if cfg!(target_os = "windows") {
        "binaries/x86_64-windows"
    } else {
        panic!("Unsupported platform")
    }
}

/// Parse modelDescription.xml from extracted FMU
pub fn parse_model_description(fmu_dir: &Path) -> Result<ModelDescription> {
    let xml_path = fmu_dir.join("modelDescription.xml");
    let xml_content = std::fs::read_to_string(&xml_path)
        .context("Failed to read modelDescription.xml")?;

    let doc = roxmltree::Document::parse(&xml_content)
        .context("Failed to parse XML")?;

    // Extract model name, GUID, etc.
    let root = doc.root_element();
    let model_name = root
        .attribute("modelName")
        .context("Missing modelName attribute")?;

    Ok(ModelDescription {
        name: model_name.to_string(),
        // ... other fields
    })
}

#[derive(Debug)]
pub struct ModelDescription {
    pub name: String,
    // Add other relevant fields as needed
}
```

**Deliverables**:
- [ ] All Protobuf messages compile and serialize correctly
- [ ] FMI type definitions with correct C ABI layout
- [ ] Utility functions tested on both Linux and Windows
- [ ] Unit tests for FMU extraction and XML parsing

---

### Phase 2: Client Library (LiaisonFMU) (3-4 weeks)

**Objective**: Port `fmi3Functions.cpp` to create a working Rust-based client FMU library

#### 2.1 Zenoh Session Management

**File**: `liaison-fmu/src/session.rs`

Port the `Placeholder` class functionality:

```rust
use zenoh::prelude::*;
use zenoh::query::{Query, Selector};
use anyhow::{Context, Result};
use std::sync::Arc;
use liaison_common::proto;

pub struct FmuSession {
    zenoh_session: Arc<zenoh::Session>,
    responder_id: String,
    instance_index: i32,
}

impl FmuSession {
    pub fn new(responder_id: String, instance_index: i32, zenoh_config: Option<String>) -> Result<Self> {
        let config = if let Some(config_path) = zenoh_config {
            zenoh::Config::from_file(&config_path)
                .context("Failed to load Zenoh config")?
        } else {
            zenoh::Config::default()
        };

        let session = zenoh::open(config)
            .wait()
            .context("Failed to open Zenoh session")?;

        Ok(FmuSession {
            zenoh_session: Arc::new(session),
            responder_id,
            instance_index,
        })
    }

    /// Send RPC request and wait for response
    pub fn call_remote_function(
        &self,
        function_name: &str,
        request: proto::Request,
    ) -> Result<proto::Response> {
        let key_expr = format!("rpc/{}/{}", self.responder_id, function_name);

        // Serialize request using prost
        let mut buf = Vec::new();
        prost::Message::encode(&request, &mut buf)?;

        // Send Zenoh query
        let replies = self.zenoh_session
            .get(&key_expr)
            .payload(buf)
            .wait()?;

        // Wait for first reply
        let reply = replies
            .recv()
            .context("No response received from server")?;

        match reply.result() {
            Ok(sample) => {
                // Deserialize response
                let response = proto::Response::decode(sample.payload().to_bytes().as_ref())
                    .context("Failed to deserialize response")?;
                Ok(response)
            }
            Err(e) => anyhow::bail!("Query failed: {:?}", e),
        }
    }
}
```

#### 2.2 FMI Function Implementations

**File**: `liaison-fmu/src/lib.rs`

Port all FMI 3.0 function implementations:

```rust
use std::ffi::{CStr, CString, c_void, c_char};
use std::os::raw::{c_uint, c_int};
use std::sync::Mutex;
use std::collections::HashMap;
use liaison_common::fmi3_types::*;
use liaison_common::proto;

mod session;
use session::FmuSession;

// Global state management
lazy_static::lazy_static! {
    static ref SESSIONS: Mutex<HashMap<*const fmi3Instance, FmuSession>> =
        Mutex::new(HashMap::new());
}

// Helper to convert instance pointer to session
fn get_session(instance: *const fmi3Instance) -> Option<&'static FmuSession> {
    let sessions = SESSIONS.lock().unwrap();
    sessions.get(&instance).map(|s| unsafe { &*(s as *const _) })
}

#[no_mangle]
pub unsafe extern "C" fn fmi3GetVersion() -> *const c_char {
    static VERSION: &[u8] = b"3.0\0";
    VERSION.as_ptr() as *const c_char
}

#[no_mangle]
pub unsafe extern "C" fn fmi3InstantiateCoSimulation(
    instance_name: *const c_char,
    instantiation_token: *const c_char,
    resource_path: *const c_char,
    visible: fmi3Boolean,
    logging_on: fmi3Boolean,
    event_mode_used: fmi3Boolean,
    early_return_allowed: fmi3Boolean,
    required_intermediate_variables: *const fmi3ValueReference,
    n_required_intermediate_variables: usize,
    instance_environment: *mut c_void,
    log_message: fmi3LogMessageCallback,
    intermediate_update: fmi3IntermediateUpdateCallback,
) -> *const fmi3Instance {
    // Load config.json from resource_path
    let resource_path_str = CStr::from_ptr(resource_path).to_str().unwrap();
    let config_path = format!("{}/config.json", resource_path_str);

    let config_content = match std::fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(_) => return std::ptr::null(),
    };

    let config: serde_json::Value = match serde_json::from_str(&config_content) {
        Ok(c) => c,
        Err(_) => return std::ptr::null(),
    };

    let responder_id = config["responderId"].as_str().unwrap().to_string();
    let zenoh_config = config["zenohConfig"].as_str().map(|s| s.to_string());

    // Create Zenoh session
    static INSTANCE_COUNTER: std::sync::atomic::AtomicI32 =
        std::sync::atomic::AtomicI32::new(0);
    let instance_idx = INSTANCE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    let session = match FmuSession::new(responder_id, instance_idx, zenoh_config) {
        Ok(s) => s,
        Err(_) => return std::ptr::null(),
    };

    // Create instance pointer (using instance_idx as unique identifier)
    let instance_ptr = instance_idx as *const fmi3Instance;

    // Build protobuf request
    let request = proto::Request {
        function: proto::FunctionId::InstantiateCoSimulation as i32,
        instance_index: instance_idx,
        // ... populate other fields from parameters
    };

    // Call remote function
    match session.call_remote_function("fmi3InstantiateCoSimulation", request) {
        Ok(response) => {
            if response.status == proto::fmi3Status::Ok as i32 {
                SESSIONS.lock().unwrap().insert(instance_ptr, session);
                instance_ptr
            } else {
                std::ptr::null()
            }
        }
        Err(_) => std::ptr::null(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn fmi3FreeInstance(instance: *const fmi3Instance) {
    if let Some(session) = get_session(instance) {
        let request = proto::Request {
            function: proto::FunctionId::FreeInstance as i32,
            instance_index: session.instance_index,
        };

        let _ = session.call_remote_function("fmi3FreeInstance", request);
    }

    SESSIONS.lock().unwrap().remove(&instance);
}

#[no_mangle]
pub unsafe extern "C" fn fmi3DoStep(
    instance: *const fmi3Instance,
    current_communication_point: fmi3Float64,
    communication_step_size: fmi3Float64,
    no_set_fmu_state_prior_to_current_point: fmi3Boolean,
    event_handling_needed: *mut fmi3Boolean,
    terminate_simulation: *mut fmi3Boolean,
    early_return: *mut fmi3Boolean,
    last_successful_time: *mut fmi3Float64,
) -> fmi3Status {
    let session = match get_session(instance) {
        Some(s) => s,
        None => return fmi3Status::Error,
    };

    let request = proto::Request {
        function: proto::FunctionId::DoStep as i32,
        instance_index: session.instance_index,
        do_step: Some(proto::DoStepRequest {
            current_communication_point,
            communication_step_size,
            no_set_fmu_state_prior_to_current_point: no_set_fmu_state_prior_to_current_point != 0,
        }),
    };

    match session.call_remote_function("fmi3DoStep", request) {
        Ok(response) => {
            if let Some(do_step_resp) = response.do_step {
                *event_handling_needed = do_step_resp.event_handling_needed as fmi3Boolean;
                *terminate_simulation = do_step_resp.terminate_simulation as fmi3Boolean;
                *early_return = do_step_resp.early_return as fmi3Boolean;
                *last_successful_time = do_step_resp.last_successful_time;
            }
            fmi3Status::from(response.status)
        }
        Err(_) => fmi3Status::Error,
    }
}

// ... implement all 30+ other FMI functions following the same pattern
```

#### 2.3 Build Configuration

**File**: `liaison-fmu/Cargo.toml`

```toml
[package]
name = "liaison-fmu"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]  # Creates shared library (.so/.dll)
name = "liaisonfmu"

[dependencies]
liaison-common = { path = "../liaison-common" }
zenoh = "1.0"
prost = "0.13"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
lazy_static = "1.5"
anyhow = "1.0"

[build-dependencies]
prost-build = "0.13"
```

**Deliverables**:
- [ ] Shared library compiles for Linux (.so) and Windows (.dll)
- [ ] All FMI 3.0 functions implemented with proper C ABI
- [ ] Zenoh session management working
- [ ] Manual testing with simple FMU succeeds

---

### Phase 3: Server Application (3-4 weeks)

**Objective**: Port `liaison.cpp` server functionality

#### 3.1 CLI Argument Parsing

**File**: `liaison-server/src/cli.rs`

```rust
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "liaison")]
#[command(about = "FMU sharing over Zenoh networks")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable debug logging
    #[arg(long, global = true)]
    pub debug: bool,

    /// Enable Zenoh-level debug logging
    #[arg(long, global = true)]
    pub debug_zenoh: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Serve an FMU over Zenoh
    Serve {
        /// Path to FMU file
        fmu_path: PathBuf,

        /// Responder ID for Zenoh key expressions
        responder_id: String,

        /// Path to Zenoh configuration file
        #[arg(long)]
        zenoh_config: Option<PathBuf>,

        /// Python environment path (for Python FMUs)
        #[arg(long)]
        python_env: Option<PathBuf>,
    },

    /// Create a Liaison FMU
    MakeFmu {
        /// Path to original FMU
        fmu_path: PathBuf,

        /// Responder ID for server
        responder_id: String,

        /// Path to Zenoh configuration file
        #[arg(long)]
        zenoh_config: Option<PathBuf>,
    },
}
```

#### 3.2 Dynamic FMU Loading

**File**: `liaison-server/src/fmu_loader.rs`

```rust
use libloading::{Library, Symbol};
use anyhow::{Context, Result};
use liaison_common::fmi3_types::*;
use std::path::Path;

pub struct LoadedFmu {
    #[allow(dead_code)]
    library: Library,  // Keep alive to prevent unloading

    // Function pointers
    pub instantiate_co_simulation: unsafe extern "C" fn(...) -> *const fmi3Instance,
    pub free_instance: unsafe extern "C" fn(*const fmi3Instance),
    pub do_step: unsafe extern "C" fn(...) -> fmi3Status,
    // ... all other function pointers
}

impl LoadedFmu {
    pub unsafe fn load(library_path: &Path) -> Result<Self> {
        let library = Library::new(library_path)
            .context("Failed to load FMU library")?;

        // Load all function symbols
        let instantiate_co_simulation: Symbol<unsafe extern "C" fn(...) -> *const fmi3Instance> =
            library.get(b"fmi3InstantiateCoSimulation")
                .context("Failed to load fmi3InstantiateCoSimulation")?;

        let free_instance: Symbol<unsafe extern "C" fn(*const fmi3Instance)> =
            library.get(b"fmi3FreeInstance")
                .context("Failed to load fmi3FreeInstance")?;

        // ... load all other functions

        Ok(LoadedFmu {
            library,
            instantiate_co_simulation: *instantiate_co_simulation,
            free_instance: *free_instance,
            // ...
        })
    }
}
```

#### 3.3 Server Main Loop

**File**: `liaison-server/src/server.rs`

```rust
use zenoh::prelude::*;
use anyhow::{Context, Result};
use liaison_common::proto;
use crate::fmu_loader::LoadedFmu;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub struct FmuServer {
    zenoh_session: Arc<zenoh::Session>,
    loaded_fmu: Arc<LoadedFmu>,
    instances: Arc<Mutex<HashMap<i32, *const fmi3Instance>>>,
    responder_id: String,
}

impl FmuServer {
    pub async fn new(
        fmu_path: &Path,
        responder_id: String,
        zenoh_config: Option<&Path>,
    ) -> Result<Self> {
        // Extract FMU
        let temp_dir = liaison_common::utils::extract_fmu(fmu_path)?;

        // Load FMU library
        let platform_dir = liaison_common::utils::get_platform_dir();
        let library_path = temp_dir.path()
            .join(platform_dir)
            .join(if cfg!(unix) { "liaisonfmu.so" } else { "liaisonfmu.dll" });

        let loaded_fmu = unsafe { LoadedFmu::load(&library_path)? };

        // Open Zenoh session
        let config = if let Some(config_path) = zenoh_config {
            zenoh::Config::from_file(config_path)?
        } else {
            zenoh::Config::default()
        };

        let session = zenoh::open(config).await?;

        Ok(FmuServer {
            zenoh_session: Arc::new(session),
            loaded_fmu: Arc::new(loaded_fmu),
            instances: Arc::new(Mutex::new(HashMap::new())),
            responder_id,
        })
    }

    pub async fn run(&self) -> Result<()> {
        // Declare queryables for each FMI function
        let key_expr = format!("rpc/{}/*", self.responder_id);

        let queryable = self.zenoh_session
            .declare_queryable(&key_expr)
            .await?;

        tracing::info!("Server started, listening on: {}", key_expr);

        // Main server loop
        loop {
            let query = queryable.recv_async().await?;
            self.handle_query(query).await;
        }
    }

    async fn handle_query(&self, query: Query) {
        // Extract function name from key expression
        let key_expr = query.key_expr().as_str();
        let function_name = key_expr.split('/').last().unwrap_or("");

        // Deserialize request
        let request = match proto::Request::decode(query.payload().unwrap().to_bytes().as_ref()) {
            Ok(req) => req,
            Err(e) => {
                tracing::error!("Failed to deserialize request: {}", e);
                return;
            }
        };

        // Dispatch to appropriate handler
        let response = match function_name {
            "fmi3InstantiateCoSimulation" => self.handle_instantiate(request),
            "fmi3FreeInstance" => self.handle_free_instance(request),
            "fmi3DoStep" => self.handle_do_step(request),
            // ... other functions
            _ => proto::Response {
                status: proto::fmi3Status::Error as i32,
                ..Default::default()
            },
        };

        // Serialize and send response
        let mut buf = Vec::new();
        prost::Message::encode(&response, &mut buf).unwrap();
        query.reply(key_expr, buf).await.unwrap();
    }

    fn handle_instantiate(&self, request: proto::Request) -> proto::Response {
        unsafe {
            let instance_ptr = (self.loaded_fmu.instantiate_co_simulation)(
                // ... convert request fields to C arguments
            );

            if !instance_ptr.is_null() {
                self.instances.lock().unwrap().insert(request.instance_index, instance_ptr);
                proto::Response {
                    status: proto::fmi3Status::Ok as i32,
                    ..Default::default()
                }
            } else {
                proto::Response {
                    status: proto::fmi3Status::Error as i32,
                    ..Default::default()
                }
            }
        }
    }

    fn handle_do_step(&self, request: proto::Request) -> proto::Response {
        let instances = self.instances.lock().unwrap();
        let instance_ptr = match instances.get(&request.instance_index) {
            Some(&ptr) => ptr,
            None => return proto::Response {
                status: proto::fmi3Status::Error as i32,
                ..Default::default()
            },
        };

        let do_step_req = request.do_step.unwrap();

        unsafe {
            let mut event_handling_needed: fmi3Boolean = 0;
            let mut terminate_simulation: fmi3Boolean = 0;
            let mut early_return: fmi3Boolean = 0;
            let mut last_successful_time: fmi3Float64 = 0.0;

            let status = (self.loaded_fmu.do_step)(
                instance_ptr,
                do_step_req.current_communication_point,
                do_step_req.communication_step_size,
                do_step_req.no_set_fmu_state_prior_to_current_point as fmi3Boolean,
                &mut event_handling_needed,
                &mut terminate_simulation,
                &mut early_return,
                &mut last_successful_time,
            );

            proto::Response {
                status: status as i32,
                do_step: Some(proto::DoStepResponse {
                    event_handling_needed: event_handling_needed != 0,
                    terminate_simulation: terminate_simulation != 0,
                    early_return: early_return != 0,
                    last_successful_time,
                }),
            }
        }
    }

    // ... implement handlers for all other FMI functions
}
```

#### 3.4 Main Entry Point

**File**: `liaison-server/src/main.rs`

```rust
mod cli;
mod server;
mod fmu_loader;
mod make_fmu;

use clap::Parser;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    // Initialize logging
    let log_level = if cli.debug {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();

    match cli.command {
        cli::Commands::Serve { fmu_path, responder_id, zenoh_config, python_env } => {
            if let Some(py_env) = python_env {
                setup_python_environment(&py_env)?;
            }

            let server = server::FmuServer::new(
                &fmu_path,
                responder_id,
                zenoh_config.as_deref(),
            ).await?;

            server.run().await?;
        }

        cli::Commands::MakeFmu { fmu_path, responder_id, zenoh_config } => {
            make_fmu::create_liaison_fmu(
                &fmu_path,
                &responder_id,
                zenoh_config.as_deref(),
            )?;
        }
    }

    Ok(())
}

fn setup_python_environment(py_env: &Path) -> Result<()> {
    // Implement Python environment setup logic
    // Similar to C++ version with LD_PRELOAD/PATH manipulation
    todo!("Implement Python environment setup")
}
```

#### 3.5 FMU Creation Tool

**File**: `liaison-server/src/make_fmu.rs`

```rust
use anyhow::{Context, Result};
use std::path::Path;
use zip::write::{FileOptions, ZipWriter};
use std::fs::File;

pub fn create_liaison_fmu(
    original_fmu_path: &Path,
    responder_id: &str,
    zenoh_config: Option<&Path>,
) -> Result<()> {
    // Extract original FMU
    let temp_dir = liaison_common::utils::extract_fmu(original_fmu_path)?;

    // Parse modelDescription.xml
    let model_desc = liaison_common::utils::parse_model_description(temp_dir.path())?;

    // Create new FMU ZIP
    let output_path = format!("{}Liaison.fmu", model_desc.name);
    let output_file = File::create(&output_path)?;
    let mut zip = ZipWriter::new(output_file);

    // Copy modelDescription.xml
    let xml_content = std::fs::read(temp_dir.path().join("modelDescription.xml"))?;
    zip.start_file("modelDescription.xml", FileOptions::default())?;
    std::io::copy(&mut xml_content.as_slice(), &mut zip)?;

    // Copy client library to appropriate platform directory
    let client_lib_path = if cfg!(target_os = "linux") {
        "target/release/libliaisonfmu.so"
    } else {
        "target/release/liaisonfmu.dll"
    };

    let platform_dir = liaison_common::utils::get_platform_dir();
    let lib_name = format!("{}.{}", model_desc.name, if cfg!(unix) { "so" } else { "dll" });
    let zip_path = format!("{}/{}", platform_dir, lib_name);

    zip.start_file(&zip_path, FileOptions::default())?;
    let mut lib_file = File::open(client_lib_path)?;
    std::io::copy(&mut lib_file, &mut zip)?;

    // Create config.json
    let config = serde_json::json!({
        "responderId": responder_id,
        "zenohConfig": zenoh_config.map(|p| p.to_string_lossy().to_string()),
    });

    zip.start_file("resources/config.json", FileOptions::default())?;
    serde_json::to_writer(&mut zip, &config)?;

    zip.finish()?;

    tracing::info!("Created Liaison FMU: {}", output_path);
    Ok(())
}
```

**Deliverables**:
- [ ] Server application compiles and runs
- [ ] CLI arguments parsed correctly
- [ ] FMU loading and dynamic linking works
- [ ] Zenoh RPC handlers functional
- [ ] `--make-fmu` command creates valid FMU packages
- [ ] End-to-end testing with FMPy succeeds

---

### Phase 4: Testing & Validation (2 weeks)

**Objective**: Achieve feature parity and validate correctness

#### 4.1 Unit Tests

**File**: `liaison-common/tests/utils_tests.rs`

```rust
#[cfg(test)]
mod tests {
    use liaison_common::utils::*;
    use std::path::PathBuf;

    #[test]
    fn test_extract_fmu() {
        let fmu_path = PathBuf::from("test_data/BouncingBall.fmu");
        let temp_dir = extract_fmu(&fmu_path).expect("Failed to extract FMU");

        assert!(temp_dir.path().join("modelDescription.xml").exists());
    }

    #[test]
    fn test_parse_model_description() {
        let fmu_path = PathBuf::from("test_data/BouncingBall.fmu");
        let temp_dir = extract_fmu(&fmu_path).unwrap();
        let model_desc = parse_model_description(temp_dir.path()).unwrap();

        assert_eq!(model_desc.name, "BouncingBall");
    }
}
```

#### 4.2 Integration Tests

Create test suite that:
1. Serves a test FMU
2. Creates Liaison FMU
3. Simulates with FMPy
4. Compares outputs to original FMU

**File**: `liaison-server/tests/integration_test.rs`

```rust
#[tokio::test]
async fn test_end_to_end_simulation() {
    // Start server in background
    let server_handle = tokio::spawn(async {
        // Start server with test FMU
    });

    // Create Liaison FMU
    // Run FMPy simulation
    // Verify outputs

    server_handle.abort();
}
```

#### 4.3 Cross-Platform Testing

- Test on Linux (Ubuntu 22.04)
- Test on Windows (MinGW)
- Validate binary sizes
- Performance benchmarking vs C++ version

**Deliverables**:
- [ ] 80%+ unit test coverage
- [ ] Integration tests pass on both platforms
- [ ] Performance within 10% of C++ version
- [ ] All FMI functions validated

---

### Phase 5: Documentation & Polish (1 week)

**Objective**: Prepare for production use

#### Tasks:

1. **Update README.md**: Replace C++ build instructions with Rust
2. **API Documentation**: Add rustdoc comments to all public APIs
3. **Migration Guide**: Document differences from C++ version
4. **Performance Comparison**: Publish benchmark results
5. **Update CI/CD**: Replace C++ workflows with Rust builds
6. **Binary Distribution**: Create release artifacts

**Deliverables**:
- [ ] Comprehensive documentation
- [ ] Published API docs (`cargo doc`)
- [ ] CI/CD producing release binaries
- [ ] Migration completed

---

## Critical Challenges & Solutions

### Challenge 1: C ABI Compatibility

**Problem**: FMI 3.0 requires exact C ABI compatibility for function signatures and data structures.

**Solution**:
```rust
// Use repr(C) for all types exposed to C
#[repr(C)]
pub struct fmi3ValueReference(pub u32);

// Use extern "C" for all exported functions
#[no_mangle]
pub unsafe extern "C" fn fmi3GetVersion() -> *const c_char { ... }

// Test ABI compatibility with C++ version
#[test]
fn test_struct_layout() {
    assert_eq!(
        std::mem::size_of::<fmi3Instance>(),
        8  // Should match C++ sizeof
    );
}
```

**Validation**: Use `cbindgen` to generate C headers and compare with original FMI headers.

---

### Challenge 2: Dynamic Library Loading

**Problem**: Must load platform-specific FMU libraries (.so/.dll) and call functions dynamically.

**Solution**: Use `libloading` crate with careful error handling:

```rust
// Handle platform-specific library extensions
let lib_name = if cfg!(target_os = "linux") {
    "libfmu.so"
} else if cfg!(target_os = "windows") {
    "fmu.dll"
} else {
    return Err(anyhow!("Unsupported platform"));
};

// Load with proper error context
let library = unsafe {
    Library::new(lib_path)
        .context(format!("Failed to load FMU library: {:?}", lib_path))?
};

// Type-safe function loading
type Fmi3GetVersionFn = unsafe extern "C" fn() -> *const c_char;
let get_version: Symbol<Fmi3GetVersionFn> = unsafe {
    library.get(b"fmi3GetVersion\0")
        .context("Symbol fmi3GetVersion not found")?
};
```

**Testing**: Validate with reference FMUs from different vendors.

---

### Challenge 3: Zenoh API Differences

**Problem**: Zenoh C++ API differs from Rust API in async patterns and ownership.

**Solution**: Embrace Rust's async model with Tokio:

```rust
// C++ uses synchronous blocking calls
// Rust uses async/await with Tokio runtime

#[tokio::main]
async fn main() -> Result<()> {
    let session = zenoh::open(config).await?;

    // Declare queryable (async)
    let queryable = session
        .declare_queryable("rpc/**")
        .await?;

    // Use async iteration
    while let Ok(query) = queryable.recv_async().await {
        tokio::spawn(async move {
            handle_query(query).await;
        });
    }
}
```

**Benefits**: Better performance with concurrent request handling, no thread-per-connection overhead.

---

### Challenge 4: Protobuf Message Handling

**Problem**: Need to maintain compatibility with existing protobuf definitions while using Rust idioms.

**Solution**: Use `prost` with custom type conversions:

```rust
// Auto-generated by prost
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Request {
    #[prost(enumeration = "FunctionId", tag = "1")]
    pub function: i32,

    #[prost(int32, tag = "2")]
    pub instance_index: i32,

    #[prost(message, optional, tag = "3")]
    pub do_step: ::core::option::Option<DoStepRequest>,
}

// Add convenience conversions
impl From<proto::fmi3Status> for fmi3Status {
    fn from(status: proto::fmi3Status) -> Self {
        match status {
            proto::fmi3Status::Ok => fmi3Status::OK,
            proto::fmi3Status::Warning => fmi3Status::Warning,
            // ...
        }
    }
}
```

**Validation**: Write bidirectional serialization tests comparing with C++ implementation.

---

### Challenge 5: Python FMU Support

**Problem**: Supporting Python-based FMUs requires environment setup and library preloading.

**Solution**: Use Rust's `std::env` and platform-specific mechanisms:

```rust
fn setup_python_environment(venv_path: &Path) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        // Find libpython in virtual environment
        let lib_path = venv_path.join("lib");
        let python_lib = find_file_matching(&lib_path, "libpython*.so")?;

        // Set LD_PRELOAD
        std::env::set_var("LD_PRELOAD", python_lib);

        // Add to LD_LIBRARY_PATH
        let ld_library_path = std::env::var("LD_LIBRARY_PATH")
            .unwrap_or_default();
        std::env::set_var(
            "LD_LIBRARY_PATH",
            format!("{}:{}", lib_path.display(), ld_library_path)
        );
    }

    #[cfg(target_os = "windows")]
    {
        // Set PATH on Windows
        let scripts_path = venv_path.join("Scripts");
        let path = std::env::var("PATH").unwrap_or_default();
        std::env::set_var("PATH", format!("{};{}", scripts_path.display(), path));
    }

    Ok(())
}
```

**Testing**: Validate with PythonFMU3 examples.

---

### Challenge 6: Cross-Compilation

**Problem**: Need to build for multiple platforms (Linux/Windows) from single source.

**Solution**: Use Cargo cross-compilation with proper toolchains:

```bash
# Install cross-compilation tool
cargo install cross

# Build for Windows from Linux
cross build --target x86_64-pc-windows-gnu --release

# Build for Linux
cargo build --target x86_64-unknown-linux-gnu --release
```

**CI Configuration**:
```yaml
# .github/workflows/build.yml
strategy:
  matrix:
    include:
      - os: ubuntu-latest
        target: x86_64-unknown-linux-gnu
      - os: windows-latest
        target: x86_64-pc-windows-gnu
```

---

## Testing Strategy

### Test Pyramid

```
                     ┌─────────────────┐
                     │  E2E Tests (5%) │  FMPy integration
                     └─────────────────┘
                  ┌──────────────────────┐
                  │ Integration Tests    │  Client-Server RPC
                  │      (20%)           │  FMU loading
                  └──────────────────────┘
            ┌──────────────────────────────────┐
            │     Unit Tests (75%)             │  Utils, Protobuf,
            │                                  │  FMI types
            └──────────────────────────────────┘
```

### Test Categories

#### 1. Unit Tests

**Scope**: Individual functions and modules

**Files**:
- `liaison-common/tests/utils_tests.rs` - FMU extraction, XML parsing
- `liaison-common/tests/proto_tests.rs` - Protobuf serialization
- `liaison-fmu/tests/session_tests.rs` - Zenoh session management

**Coverage Goal**: 80%+ line coverage

**Tools**:
```bash
cargo tarpaulin --all-features --workspace --out Html
```

#### 2. Integration Tests

**Scope**: Component interaction (client ↔ server)

**Test Scenarios**:
1. Server starts and accepts connections
2. Client FMU instantiates successfully
3. RPC calls complete with correct responses
4. Multiple concurrent instances
5. Error handling (network failures, invalid requests)

**Tools**:
- `testcontainers` for Zenoh router
- Mock FMU implementations

#### 3. End-to-End Tests

**Scope**: Full simulation workflow

**Test Scenarios**:
1. **BouncingBall FMU**: Simple physics simulation
   - Create Liaison FMU
   - Start server
   - Simulate with FMPy
   - Validate output matches original FMU

2. **Multi-instance**: Multiple FMU instances
3. **Cross-platform**: Linux client ↔ Windows server
4. **TLS/mTLS**: Secure communication

**Success Criteria**:
- Output values match within 0.01% tolerance
- Simulation completes in comparable time to original FMU

#### 4. Performance Tests

**Benchmarks**:
1. **Latency**: Time per FMI function call
2. **Throughput**: Requests per second
3. **Memory**: Resident set size during simulation
4. **Binary Size**: Executable and library sizes

**Tools**:
```bash
cargo bench
```

**Baseline Comparison**:
| Metric | C++ Version | Rust Target | Notes |
|--------|-------------|-------------|-------|
| RPC Latency | ~X ms | < 1.1X ms | Within 10% |
| Binary Size (server) | ~Y MB | < 0.9Y MB | Smaller expected |
| Binary Size (client) | ~Z MB | < 0.9Z MB | Smaller expected |
| Memory (1 instance) | ~A MB | < 1.1A MB | Within 10% |

#### 5. Compatibility Tests

**FMU Compatibility**:
- Reference FMUs (FMI Standard)
- PythonFMU3
- Commercial FMUs (if available)

**Platform Compatibility**:
- Ubuntu 22.04 (glibc 2.35)
- Windows 10/11 (MinGW)

**Simulator Compatibility**:
- FMPy 0.3.20
- Other FMI 3.0 simulators

### Continuous Integration

**GitHub Actions Workflow**:

```yaml
name: Rust CI

on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test --all-features --workspace
      - name: Run clippy
        run: cargo clippy -- -D warnings
      - name: Check formatting
        run: cargo fmt -- --check

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin
      - name: Generate coverage
        run: cargo tarpaulin --out Xml
      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

---

## Build & Deployment

### Development Build

```bash
# Clone repository
git clone https://github.com/RISE-Maritime/liaison-rs
cd liaison-rs

# Build all components
cargo build --workspace

# Run tests
cargo test --all-features --workspace

# Build release
cargo build --release --workspace
```

### Release Build

**Optimization Settings** (`Cargo.toml`):

```toml
[profile.release]
opt-level = 3           # Maximum optimizations
lto = true              # Link-time optimization
codegen-units = 1       # Better optimization, slower compile
strip = true            # Remove debug symbols
panic = 'abort'         # Smaller binaries
```

**Binary Sizes (estimated)**:
- Server: 3-5 MB (vs 8-10 MB C++)
- Client library: 2-3 MB (vs 5-7 MB C++)

### Cross-Compilation

**Linux → Windows**:
```bash
# Install MinGW toolchain
rustup target add x86_64-pc-windows-gnu

# Build for Windows
cargo build --release --target x86_64-pc-windows-gnu
```

**Musl (static Linux binaries)**:
```bash
# Install musl target
rustup target add x86_64-unknown-linux-musl

# Build fully static binary
cargo build --release --target x86_64-unknown-linux-musl
```

### Packaging

**GitHub Release Workflow**:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*.*.*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact: liaison-linux.tar.gz
          - os: windows-latest
            target: x86_64-pc-windows-gnu
            artifact: liaison-windows.zip
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - name: Build release
        run: cargo build --release --target ${{ matrix.target }}
      - name: Package artifacts
        run: |
          # Package binaries
      - name: Upload artifacts
        uses: actions/upload-artifact@v3

  release:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - name: Create GitHub release
        uses: softprops/action-gh-release@v1
```

**Distribution**:
1. GitHub Releases: Pre-built binaries for Linux/Windows
2. Crates.io: Published as Rust crate (optional)
3. Docker: Container images with server

### Docker Image

**Dockerfile**:
```dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/liaison /usr/local/bin/

ENTRYPOINT ["liaison"]
```

**Usage**:
```bash
docker run -v /path/to/fmus:/fmus liaison \
    serve /fmus/BouncingBall.fmu fmus/bouncingball
```

---

## Timeline & Resource Estimates

### Summary

| Phase | Duration | Effort (person-days) | Dependencies |
|-------|----------|---------------------|--------------|
| Phase 0: Preparation | 1 week | 3-5 | None |
| Phase 1: Core Infrastructure | 2-3 weeks | 10-15 | Phase 0 |
| Phase 2: Client Library | 3-4 weeks | 15-20 | Phase 1 |
| Phase 3: Server Application | 3-4 weeks | 15-20 | Phase 1 |
| Phase 4: Testing & Validation | 2 weeks | 10-12 | Phase 2, 3 |
| Phase 5: Documentation & Polish | 1 week | 3-5 | Phase 4 |
| **Total** | **12-15 weeks** | **56-77 days** | |

### Detailed Breakdown

#### Phase 0: Preparation (1 week)
- **Effort**: 3-5 person-days
- **Resources**: 1 developer with Rust experience
- **Deliverables**: Project structure, dependency validation, CI setup

#### Phase 1: Core Infrastructure (2-3 weeks)
- **Effort**: 10-15 person-days
- **Resources**: 1 developer (Rust + FMI knowledge)
- **Critical Path**: Protobuf definitions, FMI type bindings
- **Risks**: FMI C ABI compatibility issues

#### Phase 2: Client Library (3-4 weeks)
- **Effort**: 15-20 person-days
- **Resources**: 1-2 developers
- **Critical Path**: Zenoh session management, FMI function implementations
- **Parallel Work**: Can start Phase 3 after Phase 1 completes

#### Phase 3: Server Application (3-4 weeks)
- **Effort**: 15-20 person-days
- **Resources**: 1-2 developers
- **Critical Path**: Dynamic library loading, RPC handlers
- **Parallel Work**: Can work in parallel with Phase 2

#### Phase 4: Testing & Validation (2 weeks)
- **Effort**: 10-12 person-days
- **Resources**: 1 developer + QA tester
- **Critical Path**: End-to-end testing, cross-platform validation
- **Dependencies**: Requires Phase 2 and 3 complete

#### Phase 5: Documentation & Polish (1 week)
- **Effort**: 3-5 person-days
- **Resources**: 1 developer + technical writer (optional)
- **Critical Path**: README updates, API docs
- **Parallel Work**: Can start documentation during earlier phases

### Team Composition

**Minimum Team**: 1 senior Rust developer with FMI/simulation background

**Optimal Team**:
- 1 senior Rust developer (lead, architecture)
- 1 mid-level Rust developer (implementation)
- 1 QA engineer (testing, validation)
- 1 technical writer (documentation, part-time)

### Milestones

| Milestone | Date | Success Criteria |
|-----------|------|-----------------|
| M1: Infrastructure Complete | Week 4 | All dependencies compile, CI green |
| M2: Client Library MVP | Week 8 | Basic FMI functions work end-to-end |
| M3: Server Application MVP | Week 12 | Full feature parity with C++ version |
| M4: Testing Complete | Week 14 | All tests passing, performance validated |
| M5: Release Ready | Week 15 | Documentation complete, binaries published |

---

## Risk Assessment

### High Priority Risks

#### Risk 1: C ABI Compatibility Issues

**Probability**: Medium (40%)
**Impact**: High
**Mitigation**:
- Use `#[repr(C)]` consistently
- Validate with `cbindgen` generated headers
- Test with multiple FMU implementations early
- Allocate buffer time in Phase 2 for debugging

**Contingency**: If incompatibilities found, create C wrapper layer as intermediate solution

---

#### Risk 2: Zenoh API Changes During Migration

**Probability**: Low (15%)
**Impact**: Medium
**Mitigation**:
- Pin Zenoh version in Cargo.toml (`zenoh = "=1.0.3"`)
- Monitor Zenoh release notes
- Maintain contact with Zenoh team

**Contingency**: Budget 3-5 days for API adaptation if version must change

---

#### Risk 3: Dynamic Library Loading Platform Issues

**Probability**: Medium (30%)
**Impact**: High
**Mitigation**:
- Test early with reference FMUs on both platforms
- Use well-maintained `libloading` crate
- Have Windows and Linux test environments ready

**Contingency**: Fall back to platform-specific implementations if needed

---

#### Risk 4: Performance Regression

**Probability**: Low (20%)
**Impact**: Medium
**Mitigation**:
- Establish baseline benchmarks from C++ version
- Profile regularly during development
- Use release builds for performance testing
- Leverage Rust's zero-cost abstractions

**Contingency**: If performance issues found, use profiling tools (`perf`, `flamegraph`) to optimize hot paths

---

### Medium Priority Risks

#### Risk 5: Python FMU Support Complexity

**Probability**: Medium (35%)
**Impact**: Medium
**Mitigation**:
- Treat as lower priority feature (can be added post-MVP)
- Leverage existing C++ implementation knowledge
- Test with PythonFMU3 examples

**Contingency**: Release without Python support initially, add in v1.1

---

#### Risk 6: Testing Infrastructure Setup

**Probability**: Low (15%)
**Impact**: Low
**Mitigation**:
- Use existing test FMUs from C++ version
- Set up CI early in Phase 0
- Automate testing as much as possible

**Contingency**: Manual testing as backup if CI issues arise

---

### Risk Matrix

```
Impact
  │
H │     R1, R3
  │
M │     R2          R4, R5
  │
L │                     R6
  │
  └─────────────────────────
      Low    Medium   High
           Probability
```

---

## Success Criteria

### Functional Requirements

- [ ] **Feature Parity**: All FMI 3.0 functions implemented (same set as C++ version)
- [ ] **CLI Compatibility**: All command-line options work identically
- [ ] **FMU Compatibility**: Works with reference FMUs and PythonFMU3
- [ ] **Platform Support**: Builds and runs on Linux (x86_64) and Windows (x86_64)
- [ ] **Zenoh Integration**: Successfully communicates over Zenoh networks
- [ ] **FMU Creation**: `make-fmu` command produces valid, functional FMUs

### Non-Functional Requirements

- [ ] **Performance**: RPC latency within 10% of C++ version
- [ ] **Binary Size**: Executable and library smaller than C++ version
- [ ] **Memory Safety**: Zero memory leaks (validated with Valgrind/ASan)
- [ ] **Test Coverage**: Minimum 80% line coverage
- [ ] **Documentation**: Complete API docs and updated README
- [ ] **CI/CD**: Automated builds for all platforms passing

### Validation Tests

#### Test 1: BouncingBall FMU Simulation
```bash
# Create Liaison FMU
./liaison make-fmu BouncingBall.fmu fmus/bouncingball

# Start server
./liaison serve BouncingBall.fmu fmus/bouncingball &

# Simulate
fmpy simulate BouncingBallLiaison.fmu --show-plot

# Verify: Simulation completes, plot shows expected bouncing behavior
```

#### Test 2: Cross-Platform Communication
```bash
# Linux server
./liaison serve BouncingBall.fmu fmus/bouncingball --zenoh-config router.json &

# Windows client
fmpy simulate BouncingBallLiaison.fmu

# Verify: Successful communication across platforms
```

#### Test 3: Performance Benchmark
```bash
# Run 1000 simulation steps, measure total time
cargo bench --bench rpc_latency

# Verify: Average latency < 1.1x C++ version
```

### Acceptance Criteria

**Phase 2 (Client Library)**:
- [ ] All 30+ FMI functions exported with correct C ABI
- [ ] Successfully instantiates and runs simple FMU
- [ ] Protobuf serialization/deserialization working
- [ ] Zenoh session manages lifecycle correctly

**Phase 3 (Server Application)**:
- [ ] CLI parses all arguments identically to C++ version
- [ ] Dynamically loads FMU libraries on Linux and Windows
- [ ] Handles multiple concurrent FMU instances
- [ ] Creates valid Liaison FMUs with `make-fmu` command

**Phase 4 (Testing)**:
- [ ] All unit tests passing (80%+ coverage)
- [ ] Integration tests passing on Linux and Windows
- [ ] End-to-end test with FMPy succeeds
- [ ] Performance benchmarks within acceptable range

**Phase 5 (Documentation)**:
- [ ] README.md updated with Rust build instructions
- [ ] All public APIs documented with rustdoc
- [ ] Migration guide published
- [ ] Release binaries available on GitHub

### Definition of Done

The Rust port is considered complete when:

1. ✅ All functional requirements met
2. ✅ All validation tests passing
3. ✅ Performance meets or exceeds C++ version
4. ✅ Documentation complete and published
5. ✅ CI/CD pipeline producing release artifacts
6. ✅ At least one external user validates functionality
7. ✅ Code review completed and approved
8. ✅ License headers and attributions correct

---

## Post-Migration Considerations

### Deprecation Strategy

**Option A: Hard Cutover**
- Replace C++ version entirely
- Archive C++ code in separate branch
- Update all documentation to Rust version only

**Option B: Gradual Transition (Recommended)**
- Maintain both versions for 3-6 months
- Mark C++ version as deprecated
- Provide clear migration path for users
- Sunset C++ after validation period

### Future Enhancements (Rust-Specific)

Once the port is complete, consider these Rust-native improvements:

1. **Async Server**: Leverage Tokio for better concurrency
   - Handle multiple concurrent instances more efficiently
   - Better resource utilization

2. **WebAssembly Support**: Compile client library to WASM
   - Enable browser-based FMU simulation
   - Cloud simulation platforms

3. **gRPC Support**: Add gRPC as alternative to Zenoh
   - Broader ecosystem compatibility
   - Standard RPC mechanisms

4. **Better Error Handling**: Rich error types
   - Structured error reporting
   - Better debugging information

5. **Configuration DSL**: Rust macros for configuration
   - Type-safe configuration
   - Compile-time validation

6. **Plugins**: Dynamic plugin system
   - Custom FMI function implementations
   - Protocol extensions

### Maintenance Plan

**Ongoing Responsibilities**:
- Dependency updates (quarterly)
- Security patches (as needed)
- Zenoh version compatibility tracking
- FMI standard updates (if FMI 4.0 released)

**Resource Requirements**:
- 1 developer, 10-20% time allocation
- Monthly dependency audits
- Quarterly performance reviews

---

## Conclusion

This comprehensive plan outlines a 12-15 week effort to port the Liaison codebase from C++ to Rust. The migration is structured in five phases, prioritizing risk mitigation through early validation and parallel development streams.

**Key Success Factors**:
1. **Early Testing**: Validate critical components (C ABI, dynamic loading) in Phase 1
2. **Parallel Development**: Server and client can be developed simultaneously after Phase 1
3. **Continuous Validation**: Regular testing against reference FMUs throughout development
4. **Performance Baseline**: Establish C++ benchmarks before starting Rust implementation

**Expected Benefits**:
- ✅ Memory safety without runtime overhead
- ✅ Better cross-platform support
- ✅ Smaller binary sizes
- ✅ Native Zenoh integration (no C++ bindings)
- ✅ Modern development experience (Cargo, rustfmt, clippy)
- ✅ Easier maintenance and future enhancements

**Recommendation**: Proceed with migration given:
- Modest codebase size (~2,500 LOC)
- Well-defined FMI API boundary
- Zenoh already written in Rust
- Active development phase (good time for major refactor)

This plan provides the roadmap to successfully complete the transition while maintaining compatibility with existing users and deployments.

---

## Appendix

### A. Dependency Licenses

Verify license compatibility before proceeding:

| Crate | License | Compatible with Apache 2.0? |
|-------|---------|----------------------------|
| zenoh | Apache 2.0 | ✅ Yes |
| prost | Apache 2.0 | ✅ Yes |
| tokio | MIT | ✅ Yes |
| serde | MIT/Apache 2.0 | ✅ Yes |
| clap | MIT/Apache 2.0 | ✅ Yes |
| anyhow | MIT/Apache 2.0 | ✅ Yes |
| tracing | MIT | ✅ Yes |
| libloading | ISC | ✅ Yes |
| zip | MIT | ✅ Yes |

All dependencies are compatible with Apache 2.0 license.

### B. Rust Version Requirements

**Minimum Supported Rust Version (MSRV)**: 1.75.0

Rationale:
- Zenoh 1.0 requires 1.75+
- Modern async/await features
- Const generics stabilized

### C. Reference Materials

- [FMI 3.0 Specification](https://fmi-standard.org/)
- [Zenoh Rust Documentation](https://docs.rs/zenoh/)
- [Prost Documentation](https://docs.rs/prost/)
- [The Rustonomicon](https://doc.rust-lang.org/nomicon/) - FFI and unsafe code
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

### D. Glossary

- **FMU**: Functional Mock-up Unit - A component that implements the FMI standard
- **FMI**: Functional Mock-up Interface - A standard for model exchange and co-simulation
- **Zenoh**: A pub/sub/query protocol for IoT and edge computing
- **RPC**: Remote Procedure Call
- **C ABI**: C Application Binary Interface - calling convention for C functions
- **Protobuf**: Protocol Buffers - Google's serialization format

---

**Document Version**: 1.0
**Last Updated**: 2025-11-07
**Author**: Claude (AI Assistant)
**Status**: Ready for Review
