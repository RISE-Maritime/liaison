# Extending the Test Framework

This guide shows how to add new tests and extend the framework.

## Adding a New Test

### Basic Test Template

```rust
#[tokio::test]
async fn test_my_new_feature() -> Result<()> {
    // 1. Create test fixture (automatically starts server with mock FMU)
    let fixture = TestFixture::new().await?;

    // 2. Prepare your message
    let msg = MyMessage {
        field1: value1,
        field2: value2,
    };

    // 3. Encode the message
    let mut payload = Vec::new();
    msg.encode(&mut payload)?;

    // 4. Send query to server
    let response: MyResponse = fixture
        .query("fmi3MyFunction", &payload)
        .await?;

    // 5. Verify the response
    assert_eq!(response.status, Status::Ok as i32);
    assert_eq!(response.some_value, expected_value);

    Ok(())
}
```

## Adding New Proto Messages

If you need to test a function that requires new message types:

1. **Check the proto file**: `/workspace/src/fmi3.proto`

2. **Add message definition** to `integration_test.rs`:

```rust
#[derive(Clone, PartialEq, prost::Message)]
struct MyNewMessage {
    #[prost(int32, tag = "1")]
    pub field1: i32,

    #[prost(string, tag = "2")]
    pub field2: String,

    #[prost(double, repeated, tag = "3")]
    pub values: Vec<f64>,
}
```

3. **Use in test**:

```rust
#[tokio::test]
async fn test_with_new_message() -> Result<()> {
    let fixture = TestFixture::new().await?;

    let msg = MyNewMessage {
        field1: 42,
        field2: "test".to_string(),
        values: vec![1.0, 2.0, 3.0],
    };

    let mut payload = Vec::new();
    msg.encode(&mut payload)?;

    let response: StatusMessage = fixture
        .query("fmi3NewFunction", &payload)
        .await?;

    assert_eq!(response.status, Status::Ok as i32);
    Ok(())
}
```

## Testing Error Conditions

### Test Invalid Input

```rust
#[tokio::test]
async fn test_invalid_value_reference() -> Result<()> {
    let fixture = TestFixture::new().await?;

    // Create instance first
    let instance_index = create_test_instance(&fixture).await?;

    // Try to get a non-existent variable
    let msg = GetFloat64InputMessage {
        instance_index,
        value_references: vec![999], // Invalid reference
        n_value_references: 1,
    };

    let mut payload = Vec::new();
    msg.encode(&mut payload)?;

    let response: GetFloat64OutputMessage = fixture
        .query("fmi3GetFloat64", &payload)
        .await?;

    // Should return error status
    assert_eq!(response.status, Status::Error as i32);

    Ok(())
}
```

### Test Timeout Scenarios

```rust
#[tokio::test]
async fn test_query_timeout() -> Result<()> {
    let fixture = TestFixture::new().await?;

    // Try query with very short timeout
    let key = fixture.key("fmi3SomeFunction");
    let payload = vec![];

    let result = tokio::time::timeout(
        Duration::from_millis(100),
        async {
            fixture.session
                .get(&key)
                .payload(payload)
                .await
        }
    ).await;

    // Should timeout
    assert!(result.is_err());

    Ok(())
}
```

## Creating Helper Functions

For tests that share common setup, create helper functions:

```rust
/// Helper to create and initialize an instance
async fn create_initialized_instance(
    fixture: &TestFixture
) -> Result<i32> {
    // Instantiate
    let instantiate_msg = InstantiateCoSimulationMessage {
        instance_name: format!("helper_instance_{}", rand::random::<u32>()),
        instantiation_token: "{12345678-1234-1234-1234-123456789012}".to_string(),
        resource_path: "file:///tmp/resources".to_string(),
        visible: false,
        logging_on: false,
        event_mode_used: false,
        early_return_allowed: false,
        required_intermediate_variables: vec![],
        n_required_intermediate_variables: 0,
    };

    let mut payload = Vec::new();
    instantiate_msg.encode(&mut payload)?;

    let response: InstanceMessage = fixture
        .query("fmi3InstantiateCoSimulation", &payload)
        .await?;

    let instance_index = response.instance_index;

    // Initialize
    let init_msg = EnterInitializationModeMessage {
        instance_index,
        tolerance_defined: false,
        tolerance: 0.0,
        start_time: 0.0,
        stop_time_defined: false,
        stop_time: 0.0,
    };

    let mut payload = Vec::new();
    init_msg.encode(&mut payload)?;

    fixture.query::<StatusMessage>("fmi3EnterInitializationMode", &payload).await?;

    // Exit initialization
    let exit_msg = InstanceMessage { instance_index };
    let mut payload = Vec::new();
    exit_msg.encode(&mut payload)?;

    fixture.query::<StatusMessage>("fmi3ExitInitializationMode", &payload).await?;

    Ok(instance_index)
}

// Use in test
#[tokio::test]
async fn test_with_helper() -> Result<()> {
    let fixture = TestFixture::new().await?;
    let instance = create_initialized_instance(&fixture).await?;

    // Now test with already initialized instance
    // ...

    Ok(())
}
```

## Testing with Custom FMUs

To test with a different FMU (not the mock):

```rust
#[tokio::test]
async fn test_with_custom_fmu() -> Result<()> {
    let custom_fmu_path = PathBuf::from("/path/to/your.fmu");
    let responder_id = format!("custom_test_{}", std::process::id());

    // Start server with custom FMU
    let server = utils::ServerHandle::start(utils::ServerConfig {
        fmu_path: custom_fmu_path,
        responder_id: responder_id.clone(),
        zenoh_config: None,
        debug: true,
    })?;

    // Create Zenoh session
    let session = utils::create_test_zenoh_session().await?;

    // Create custom fixture
    let fixture = TestFixture {
        server,
        session,
        responder_id,
    };

    // Run your tests
    // ...

    Ok(())
}
```

## Performance Testing

### Measure Query Latency

```rust
#[tokio::test]
async fn test_query_performance() -> Result<()> {
    let fixture = TestFixture::new().await?;
    let instance = create_initialized_instance(&fixture).await?;

    let mut latencies = Vec::new();

    for _ in 0..100 {
        let start = std::time::Instant::now();

        let msg = GetFloat64InputMessage {
            instance_index: instance,
            value_references: vec![0, 1],
            n_value_references: 2,
        };

        let mut payload = Vec::new();
        msg.encode(&mut payload)?;

        let _response: GetFloat64OutputMessage = fixture
            .query("fmi3GetFloat64", &payload)
            .await?;

        latencies.push(start.elapsed());
    }

    let avg_latency = latencies.iter().sum::<Duration>() / latencies.len() as u32;
    let max_latency = latencies.iter().max().unwrap();

    println!("Average latency: {:?}", avg_latency);
    println!("Max latency: {:?}", max_latency);

    // Assert performance requirements
    assert!(avg_latency < Duration::from_millis(50));

    Ok(())
}
```

### Throughput Testing

```rust
#[tokio::test]
async fn test_high_throughput() -> Result<()> {
    let fixture = TestFixture::new().await?;
    let instance = create_initialized_instance(&fixture).await?;

    let start = std::time::Instant::now();
    let num_queries = 1000;

    for _ in 0..num_queries {
        let msg = GetFloat64InputMessage {
            instance_index: instance,
            value_references: vec![0],
            n_value_references: 1,
        };

        let mut payload = Vec::new();
        msg.encode(&mut payload)?;

        let _: GetFloat64OutputMessage = fixture
            .query("fmi3GetFloat64", &payload)
            .await?;
    }

    let elapsed = start.elapsed();
    let qps = num_queries as f64 / elapsed.as_secs_f64();

    println!("Throughput: {:.0} queries/sec", qps);

    Ok(())
}
```

## Parallel Testing

Test multiple servers simultaneously:

```rust
#[tokio::test]
async fn test_multiple_servers() -> Result<()> {
    let fmu_path = utils::ensure_mock_fmu_built()?;

    // Start multiple servers
    let mut servers = Vec::new();
    for i in 0..3 {
        let responder_id = format!("test_server_{}_{}", i, std::process::id());
        let server = utils::ServerHandle::start(utils::ServerConfig {
            fmu_path: fmu_path.clone(),
            responder_id: responder_id.clone(),
            zenoh_config: None,
            debug: false,
        })?;
        servers.push((server, responder_id));
    }

    let session = utils::create_test_zenoh_session().await?;

    // Query all servers in parallel
    let mut handles = Vec::new();
    for (_server, responder_id) in &servers {
        let session = session.clone();
        let responder_id = responder_id.clone();

        let handle = tokio::spawn(async move {
            let key = format!("liaison/{}/fmi3GetVersion", responder_id);
            let replies = session.get(&key).await.unwrap();

            while let Ok(reply) = replies.recv_async().await {
                if let Ok(sample) = reply.result() {
                    let version = String::from_utf8(
                        sample.payload().to_bytes().to_vec()
                    ).unwrap();
                    return version;
                }
            }
            panic!("No reply");
        });

        handles.push(handle);
    }

    // Wait for all queries
    let results = futures::future::join_all(handles).await;

    // Verify all succeeded
    for result in results {
        assert_eq!(result.unwrap(), "3.0");
    }

    Ok(())
}
```

## Best Practices

1. **Use unique names**: Generate unique instance names to avoid conflicts
   ```rust
   let name = format!("test_{}_{}", test_name, std::process::id());
   ```

2. **Add context**: Use `.context()` for better error messages
   ```rust
   fixture.query("fmi3DoStep", &payload)
       .await
       .context("Failed to execute doStep")?;
   ```

3. **Clean up**: Use RAII patterns (Drop trait) for cleanup
   ```rust
   struct TestContext {
       fixture: TestFixture,
       instance: i32,
   }

   impl Drop for TestContext {
       fn drop(&mut self) {
           // Cleanup code
       }
   }
   ```

4. **Test isolation**: Each test should be independent
   - Don't share state between tests
   - Use unique responder IDs
   - Clean up resources

5. **Document tests**: Add comments explaining what's being tested
   ```rust
   /// Tests that the server correctly handles multiple simultaneous doStep calls
   /// from different instances without race conditions.
   #[tokio::test]
   async fn test_concurrent_do_step() -> Result<()> {
       // ...
   }
   ```

## Debugging Tests

### Enable Logging

```bash
# Server logs
RUST_LOG=liaison_server=debug cargo test --test integration_test -- --nocapture

# All logs
RUST_LOG=debug cargo test --test integration_test -- --nocapture

# Specific test
RUST_LOG=debug cargo test --test integration_test test_name -- --nocapture
```

### Print Debugging

```rust
#[tokio::test]
async fn test_with_debug_output() -> Result<()> {
    println!("Starting test...");

    let fixture = TestFixture::new().await?;
    println!("Created fixture");

    let msg = /* ... */;
    println!("Message: {:?}", msg);

    let response = fixture.query("fmi3Function", &payload).await?;
    println!("Response: {:?}", response);

    Ok(())
}
```

Run with: `cargo test --test integration_test test_with_debug_output -- --nocapture`

### Attach Debugger

For GDB/LLDB debugging:

```bash
# Build with debug symbols
cargo build --bin liaison

# Find the test binary
TEST_BINARY=$(find target/debug/deps -name "integration_test-*" -type f | head -1)

# Run under debugger
gdb --args $TEST_BINARY test_name --nocapture
```

## Contributing Tests

When contributing new tests:

1. Follow existing patterns
2. Add documentation
3. Update this guide if adding new patterns
4. Ensure tests pass in CI
5. Keep tests fast (< 30 seconds per test)
