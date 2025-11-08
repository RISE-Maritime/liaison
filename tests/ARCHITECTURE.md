# Test Framework Architecture

Visual overview of the test framework components and data flow.

## Component Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        Test Framework                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌─────────────────┐                                            │
│  │ integration_test│                                            │
│  │     .rs         │  Test Cases:                               │
│  │                 │  • test_server_starts                      │
│  │  [5 test cases] │  • test_get_version                        │
│  │                 │  • test_instantiate_cosimulation           │
│  │  [Proto msgs]   │  • test_full_simulation_cycle              │
│  │                 │  • test_set_and_get_variables              │
│  └────────┬────────┘  • test_multiple_instances                 │
│           │                                                       │
│           │ uses                                                 │
│           ↓                                                       │
│  ┌─────────────────┐                                            │
│  │ utils/mod.rs    │  Components:                               │
│  │                 │  • ServerHandle (lifecycle)                │
│  │  [Test Utils]   │  • TestFixture (env)                       │
│  │                 │  • Helper functions                        │
│  └────────┬────────┘                                            │
│           │                                                       │
│           │ manages                                              │
│           ↓                                                       │
│  ┌─────────────────────────────────────────┐                    │
│  │         fixtures/                       │                    │
│  │                                         │                    │
│  │  mock_fmu.c + modelDescription.xml     │                    │
│  │  build_mock_fmu.sh                     │                    │
│  │                                         │                    │
│  │  → builds → MockFMU.fmu                │                    │
│  └─────────────────────────────────────────┘                    │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

## Data Flow Diagram

```
Test Execution Flow:
────────────────────

┌──────────┐
│  Test    │
│ Starts   │
└────┬─────┘
     │
     ↓
┌────────────────────┐
│ TestFixture::new() │
└────┬───────────────┘
     │
     ├─→ 1. ensure_mock_fmu_built()
     │   └─→ build_mock_fmu.sh
     │       └─→ gcc → MockFMU.fmu
     │
     ├─→ 2. ServerHandle::start()
     │   └─→ std::process::Command
     │       └─→ liaison serve MockFMU.fmu
     │
     └─→ 3. create_test_zenoh_session()
         └─→ zenoh::open()

┌────────────────────┐
│  Server Running    │
│  Zenoh Connected   │
└────┬───────────────┘
     │
     ↓ fixture.query(function, payload)
     │
┌────────────────────┐
│  1. Encode Proto   │
│     message        │
└────┬───────────────┘
     │
     ↓
┌────────────────────┐
│  2. Send Zenoh     │
│     Query          │
│                    │
│  Key: liaison/     │
│       responder/   │
│       function     │
└────┬───────────────┘
     │
     ↓ Network (Zenoh)
     │
┌────────────────────┐
│  3. Server         │
│     Processes      │
│                    │
│  • Load FMU        │
│  • Call function   │
│  • Return result   │
└────┬───────────────┘
     │
     ↓ Response
     │
┌────────────────────┐
│  4. Decode Proto   │
│     response       │
└────┬───────────────┘
     │
     ↓
┌────────────────────┐
│  5. Assert         │
│     Results        │
└────┬───────────────┘
     │
     ↓
┌────────────────────┐
│  Test Complete     │
└────┬───────────────┘
     │
     ↓ Drop TestFixture
     │
┌────────────────────┐
│ ServerHandle::drop │
│                    │
│ • Send SIGTERM     │
│ • Wait for exit    │
│ • Force kill if    │
│   needed           │
└────────────────────┘
```

## Class/Module Structure

```
integration_test.rs
├── Proto Message Types
│   ├── StatusMessage
│   ├── InstanceMessage
│   ├── InstantiateCoSimulationMessage
│   ├── EnterInitializationModeMessage
│   ├── DoStepMessage
│   ├── GetFloat64InputMessage
│   ├── GetFloat64OutputMessage
│   └── SetFloat64InputMessage
│
├── TestFixture (struct)
│   ├── server: ServerHandle
│   ├── session: zenoh::Session
│   ├── responder_id: String
│   │
│   └── Methods:
│       ├── new() -> Result<Self>
│       ├── key(function) -> String
│       └── query<T>(function, payload) -> Result<T>
│
└── Test Functions
    ├── test_server_starts()
    ├── test_get_version()
    ├── test_instantiate_cosimulation()
    ├── test_full_simulation_cycle()
    ├── test_set_and_get_variables()
    └── test_multiple_instances()

utils/mod.rs
├── ServerConfig (struct)
│   ├── fmu_path: PathBuf
│   ├── responder_id: String
│   ├── zenoh_config: Option<PathBuf>
│   └── debug: bool
│
├── ServerHandle (struct)
│   ├── process: Child
│   ├── responder_id: String
│   │
│   └── Methods:
│       ├── start(config) -> Result<Self>
│       ├── is_running() -> bool
│       ├── stop() -> Result<()>
│       └── drop() [RAII cleanup]
│
└── Helper Functions
    ├── find_server_binary() -> Result<PathBuf>
    ├── get_workspace_dir() -> Result<PathBuf>
    ├── get_fixtures_dir() -> Result<PathBuf>
    ├── ensure_mock_fmu_built() -> Result<PathBuf>
    ├── wait_for(condition, timeout) -> Result<()>
    └── create_test_zenoh_session() -> Result<Session>

fixtures/
├── mock_fmu.c
│   ├── MockFMUInstance (struct)
│   │   ├── instanceName: char*
│   │   ├── time: fmi3Float64
│   │   ├── counter: fmi3Float64
│   │   ├── stepSize: fmi3Float64
│   │   └── callbacks
│   │
│   └── FMI Functions (10 functions)
│       ├── fmi3GetVersion()
│       ├── fmi3InstantiateCoSimulation()
│       ├── fmi3EnterInitializationMode()
│       ├── fmi3ExitInitializationMode()
│       ├── fmi3EnterStepMode()
│       ├── fmi3DoStep()
│       ├── fmi3GetFloat64()
│       ├── fmi3SetFloat64()
│       ├── fmi3Terminate()
│       └── fmi3FreeInstance()
│
├── modelDescription.xml
│   └── FMU Metadata
│       ├── Model info
│       ├── Variables (3)
│       └── Structure
│
└── build_mock_fmu.sh
    └── Build pipeline:
        ├── 1. Compile C → shared lib
        ├── 2. Create FMU structure
        ├── 3. Copy files
        └── 4. Zip → .fmu
```

## State Machine: Server Lifecycle

```
┌─────────┐
│  Start  │
└────┬────┘
     │
     ↓
┌────────────────┐
│ Build FMU      │
│ (if needed)    │
└────┬───────────┘
     │
     ↓
┌────────────────┐
│ Spawn Server   │
│ Process        │
└────┬───────────┘
     │
     ↓
┌────────────────┐
│ Wait 2s for    │
│ Initialization │
└────┬───────────┘
     │
     ↓
┌────────────────┐
│ Server Ready   │◄────────────┐
│                │             │
│ [Running]      │             │
│                │   Keep-alive│
└────┬───────────┘             │
     │                         │
     ↓ Test completes          │
     │                         │
┌────────────────┐             │
│ Drop Handle    │             │
└────┬───────────┘             │
     │                         │
     ↓                         │
┌────────────────┐             │
│ Send SIGTERM   │             │
│ (Unix)         │             │
└────┬───────────┘             │
     │                         │
     ↓                         │
┌────────────────┐             │
│ Wait 500ms     │             │
└────┬───────────┘             │
     │                         │
     ↓                         │
  Still running? ──No──> Exit Success
     │
     │Yes
     ↓
┌────────────────┐
│ Force Kill     │
└────┬───────────┘
     │
     ↓
┌────────────────┐
│ Wait for Exit  │
└────┬───────────┘
     │
     ↓
┌────────────────┐
│ Cleanup Done   │
└────────────────┘
```

## Communication Protocol

```
Test Process                     Zenoh Network                Server Process
─────────────                    ──────────────               ──────────────

┌──────────┐                                                  ┌──────────┐
│ Encode   │                                                  │  Server  │
│ Proto    │                                                  │ Listening│
│ Message  │                                                  │          │
└────┬─────┘                                                  │ Key:     │
     │                                                        │ liaison/ │
     │ query("liaison/responder/function", payload)          │ responder│
     │                                                        │ /        │
     ├──────────────────────────┐                            │ function │
     │                          │                            └────┬─────┘
     ↓                          ↓                                 │
┌──────────┐            ┌────────────┐                           │
│ Session  │            │  Zenoh     │                           │
│ get()    │───────────>│  Router    │──────────────────────────>│
└────┬─────┘            │            │                           │
     │                  │ [Matching] │                           │
     │                  │ [Routing]  │                           │
     │                  └────┬───────┘                           │
     │                       │                                   ↓
     │                       │                           ┌───────────────┐
     │                       │                           │ Decode Proto  │
     │                       │                           │ Call FMU      │
     │                       │                           │ Encode Reply  │
     │                       │                           └───────┬───────┘
     │                       │                                   │
     │                       │<──────────────────────────────────┤
     │                       │           Response                │
     ↓                       ↓                                   │
┌──────────┐           ┌────────────┐                           │
│ Receive  │<──────────│   Zenoh    │                           │
│ Reply    │           │   Reply    │                           │
└────┬─────┘           └────────────┘                           │
     │                                                           │
     ↓                                                           │
┌──────────┐                                                    │
│ Decode   │                                                    │
│ Proto    │                                                    │
│ Response │                                                    │
└────┬─────┘                                                    │
     │                                                           │
     ↓                                                           │
┌──────────┐                                                    │
│ Assert   │                                                    │
│ Results  │                                                    │
└──────────┘                                                    ↓
                                                        [Continue serving]
```

## File System Layout

```
/workspace/
├── Cargo.toml                    [Workspace config with test deps]
│
├── liaison-server/               [Server implementation]
│   ├── src/
│   │   ├── main.rs              [Server entry point]
│   │   ├── server.rs            [Server logic]
│   │   ├── queryable_handlers.rs[FMI handlers]
│   │   └── ...
│   └── Cargo.toml
│
├── liaison-fmi/                  [Client library]
│   ├── src/
│   │   ├── lib.rs
│   │   ├── fmi3.rs              [FMI functions]
│   │   └── ...
│   └── Cargo.toml
│
├── tests/                        [Integration tests - NEW]
│   ├── integration_test.rs      [Test suite]
│   ├── utils/
│   │   └── mod.rs              [Test utilities]
│   ├── fixtures/
│   │   ├── mock_fmu.c          [Mock FMU source]
│   │   ├── modelDescription.xml [FMU metadata]
│   │   ├── build_mock_fmu.sh   [Build script]
│   │   └── build/              [Generated]
│   │       └── MockFMU.fmu
│   ├── README.md               [Documentation]
│   ├── QUICKSTART.md           [Quick start]
│   ├── EXTENDING.md            [Extension guide]
│   ├── CHEATSHEET.md           [Quick reference]
│   ├── ARCHITECTURE.md         [This file]
│   ├── Makefile                [Build automation]
│   └── .gitignore              [Ignore artifacts]
│
└── target/
    └── debug/
        └── liaison             [Server binary - tested]
```

## Test Execution Timeline

```
Time    Action
────    ──────
0.0s    Test starts
0.1s    ├─ Check if MockFMU.fmu exists
0.2s    ├─ Build FMU (if needed) [~1s first time]
1.2s    ├─ Spawn server process
1.3s    ├─ Create Zenoh session
3.3s    └─ Wait 2s for server init

3.3s    Server ready, test begins

        [Test execution]
        ├─ Query 1: ~10-50ms
        ├─ Query 2: ~10-50ms
        ├─ Query 3: ~10-50ms
        └─ ...

5.0s    Test complete
5.0s    ├─ Drop TestFixture
5.0s    ├─ Send SIGTERM
5.5s    ├─ Server exits
5.5s    └─ Cleanup complete
```

## Scalability

```
Single Test:
  1 Server → 1 FMU → N instances

Multiple Tests (parallel):
  Test 1: Server A (responder_A) → FMU → instances
  Test 2: Server B (responder_B) → FMU → instances
  Test 3: Server C (responder_C) → FMU → instances

  → No conflicts (unique responder IDs)
  → Safe parallel execution
  → Isolated failures
```

## Error Handling Flow

```
┌──────────────┐
│ Test Action  │
└──────┬───────┘
       │
       ↓
    Success? ──Yes──> Continue
       │
       No
       ↓
┌──────────────┐
│ anyhow::Error│
└──────┬───────┘
       │
       ├─→ .context("Additional info")
       │
       ├─→ Propagate with ?
       │
       ↓
┌──────────────┐
│ Test Fails   │
│              │
│ • Error msg  │
│ • Context    │
│ • Backtrace  │
└──────┬───────┘
       │
       ↓
┌──────────────┐
│ Cleanup      │
│ (Drop trait) │
│              │
│ • Kill server│
│ • Close conn │
└──────────────┘
```

## Key Architectural Decisions

### 1. RAII Pattern for Cleanup
- ServerHandle uses Drop trait
- Automatic cleanup on scope exit
- No manual cleanup needed
- Guaranteed cleanup even on panic

### 2. Unique Responder IDs
- Include process ID in responder name
- Prevents conflicts in parallel tests
- Isolated test environments

### 3. Embedded Proto Definitions
- Proto messages defined in test file
- No external proto compilation for tests
- Simpler build process
- Self-contained tests

### 4. Mock FMU Auto-Build
- Build FMU on first test run
- Cache for subsequent runs
- No manual build step
- Platform-specific builds

### 5. Async Runtime (Tokio)
- Required for Zenoh async API
- Efficient concurrent I/O
- Timeout support
- Future-ready for parallel tests

## Performance Characteristics

| Operation | Typical Duration |
|-----------|------------------|
| FMU Build (first time) | ~1 second |
| Server startup | ~2 seconds |
| Zenoh session create | ~0.5 seconds |
| Query round-trip | ~10-50 ms |
| Full test (6 tests) | ~20-30 seconds |

## Dependencies Graph

```
integration_test.rs
  ├─> utils/mod.rs
  │     ├─> anyhow
  │     ├─> std::process
  │     └─> zenoh
  │
  ├─> prost (proto encoding)
  ├─> tokio (async runtime)
  └─> zenoh (communication)

Server Binary (liaison)
  ├─> FMU (MockFMU.fmu)
  │     └─> mock_fmu.c (compiled)
  │
  └─> Zenoh network

MockFMU.fmu
  └─> libc (system libs)
```

This architecture provides a robust, maintainable, and extensible test framework for the Liaison FMI system.
