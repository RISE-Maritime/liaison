// Integration tests for Liaison client-server communication
//
// These tests verify the full end-to-end interaction between:
// 1. A liaison server serving an FMU
// 2. A client using the liaison-fmi library to interact with the FMU
//
// Test strategy:
// - Start a liaison server with a mock FMU
// - Use Zenoh to communicate with the server
// - Call FMI functions through the protocol
// - Verify responses match expected behavior

use anyhow::{Context, Result};
use prost::Message;
use std::time::Duration;
use zenoh::query::QueryTarget;

mod utils;

// Import proto definitions (we'll need to share these)
// For now, we'll define minimal structures for testing

#[derive(Clone, PartialEq, prost::Message)]
struct StatusMessage {
    #[prost(enumeration = "Status", tag = "1")]
    pub status: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, prost::Enumeration)]
#[repr(i32)]
enum Status {
    Ok = 0,
    Warning = 1,
    Discard = 2,
    Error = 3,
    Fatal = 4,
}

#[derive(Clone, PartialEq, prost::Message)]
struct InstantiateCoSimulationMessage {
    #[prost(string, tag = "1")]
    pub instance_name: String,
    #[prost(string, tag = "2")]
    pub instantiation_token: String,
    #[prost(string, tag = "3")]
    pub resource_path: String,
    #[prost(bool, tag = "4")]
    pub visible: bool,
    #[prost(bool, tag = "5")]
    pub logging_on: bool,
    #[prost(bool, tag = "6")]
    pub event_mode_used: bool,
    #[prost(bool, tag = "7")]
    pub early_return_allowed: bool,
    #[prost(int32, repeated, tag = "8")]
    pub required_intermediate_variables: Vec<i32>,
    #[prost(int32, tag = "9")]
    pub n_required_intermediate_variables: i32,
}

#[derive(Clone, PartialEq, prost::Message)]
struct InstanceMessage {
    #[prost(int32, tag = "1")]
    pub instance_index: i32,
}

#[derive(Clone, PartialEq, prost::Message)]
struct EnterInitializationModeMessage {
    #[prost(int32, tag = "1")]
    pub instance_index: i32,
    #[prost(bool, tag = "2")]
    pub tolerance_defined: bool,
    #[prost(double, tag = "3")]
    pub tolerance: f64,
    #[prost(double, tag = "4")]
    pub start_time: f64,
    #[prost(bool, tag = "5")]
    pub stop_time_defined: bool,
    #[prost(double, tag = "6")]
    pub stop_time: f64,
}

#[derive(Clone, PartialEq, prost::Message)]
struct DoStepMessage {
    #[prost(int32, tag = "1")]
    pub instance_index: i32,
    #[prost(double, tag = "2")]
    pub current_communication_point: f64,
    #[prost(double, tag = "3")]
    pub communication_step_size: f64,
    #[prost(bool, tag = "4")]
    pub no_set_fmu_state_prior_to_current_point: bool,
    #[prost(bool, tag = "5")]
    pub event_handling_needed: bool,
    #[prost(bool, tag = "6")]
    pub terminate_simulation: bool,
    #[prost(bool, tag = "7")]
    pub early_return: bool,
    #[prost(double, tag = "8")]
    pub last_successful_time: f64,
}

#[derive(Clone, PartialEq, prost::Message)]
struct GetFloat64InputMessage {
    #[prost(int32, tag = "1")]
    pub instance_index: i32,
    #[prost(int32, repeated, tag = "2")]
    pub value_references: Vec<i32>,
    #[prost(int32, tag = "3")]
    pub n_value_references: i32,
}

#[derive(Clone, PartialEq, prost::Message)]
struct GetFloat64OutputMessage {
    #[prost(double, repeated, tag = "1")]
    pub values: Vec<f64>,
    #[prost(int32, tag = "2")]
    pub n_values: i32,
    #[prost(enumeration = "Status", tag = "3")]
    pub status: i32,
}

#[derive(Clone, PartialEq, prost::Message)]
struct SetFloat64InputMessage {
    #[prost(int32, tag = "1")]
    pub instance_index: i32,
    #[prost(int32, repeated, tag = "2")]
    pub value_references: Vec<i32>,
    #[prost(int32, tag = "3")]
    pub n_value_references: i32,
    #[prost(double, repeated, tag = "4")]
    pub values: Vec<f64>,
    #[prost(int32, tag = "5")]
    pub n_values: i32,
}

/// Test fixture that manages server lifecycle
struct TestFixture {
    server: utils::ServerHandle,
    session: zenoh::Session,
    responder_id: String,
}

impl TestFixture {
    async fn new() -> Result<Self> {
        // Ensure mock FMU is built
        let fmu_path = utils::ensure_mock_fmu_built()?;

        let responder_id = format!("test_server_{}", std::process::id());

        // Start the server
        let server = utils::ServerHandle::start(utils::ServerConfig {
            fmu_path,
            responder_id: responder_id.clone(),
            zenoh_config: None,
            debug: true,
        })?;

        // Create Zenoh session
        let session = utils::create_test_zenoh_session().await?;

        // Wait a bit for everything to stabilize
        tokio::time::sleep(Duration::from_millis(500)).await;

        Ok(TestFixture {
            server,
            session,
            responder_id,
        })
    }

    fn key(&self, function: &str) -> String {
        format!("liaison/{}/{}", self.responder_id, function)
    }

    async fn query<T: Message + Default>(&self, function: &str, payload: &[u8]) -> Result<T> {
        let key = self.key(function);

        let replies = self
            .session
            .get(&key)
            .payload(payload.to_vec())
            .target(QueryTarget::BestMatching)
            .await
            .context("Failed to send query")?;

        // Get the first reply
        let reply = tokio::time::timeout(Duration::from_secs(5), async {
            while let Ok(reply) = replies.recv_async().await {
                if let Ok(sample) = reply.result() {
                    return Ok(sample.payload().to_bytes().to_vec());
                }
            }
            Err(anyhow::anyhow!("No valid reply received"))
        })
        .await
        .context("Timeout waiting for reply")??;

        // Decode the response
        T::decode(&reply[..]).context("Failed to decode response")
    }
}

#[tokio::test]
async fn test_server_starts() -> Result<()> {
    let _fixture = TestFixture::new().await?;
    // If we get here, the server started successfully
    Ok(())
}

#[tokio::test]
async fn test_get_version() -> Result<()> {
    let fixture = TestFixture::new().await?;

    let key = fixture.key("fmi3GetVersion");

    let replies = fixture
        .session
        .get(&key)
        .await
        .context("Failed to query version")?;

    let reply = tokio::time::timeout(Duration::from_secs(5), async {
        while let Ok(reply) = replies.recv_async().await {
            if let Ok(sample) = reply.result() {
                let payload = sample.payload().to_bytes();
                let version = String::from_utf8(payload.to_vec())?;
                return Ok(version);
            }
        }
        Err(anyhow::anyhow!("No valid reply received"))
    })
    .await
    .context("Timeout waiting for reply")??;

    assert_eq!(reply, "3.0");
    Ok(())
}

#[tokio::test]
async fn test_instantiate_cosimulation() -> Result<()> {
    let fixture = TestFixture::new().await?;

    let msg = InstantiateCoSimulationMessage {
        instance_name: "test_instance".to_string(),
        instantiation_token: "{12345678-1234-1234-1234-123456789012}".to_string(),
        resource_path: "file:///tmp/resources".to_string(),
        visible: false,
        logging_on: true,
        event_mode_used: false,
        early_return_allowed: false,
        required_intermediate_variables: vec![],
        n_required_intermediate_variables: 0,
    };

    let mut payload = Vec::new();
    msg.encode(&mut payload)?;

    let response: InstanceMessage = fixture
        .query("fmi3InstantiateCoSimulation", &payload)
        .await?;

    // Should get a valid instance index (>= 0)
    assert!(response.instance_index >= 0);
    println!("Created instance with index: {}", response.instance_index);

    Ok(())
}

#[tokio::test]
async fn test_full_simulation_cycle() -> Result<()> {
    let fixture = TestFixture::new().await?;

    // 1. Instantiate
    let instantiate_msg = InstantiateCoSimulationMessage {
        instance_name: "test_simulation".to_string(),
        instantiation_token: "{12345678-1234-1234-1234-123456789012}".to_string(),
        resource_path: "file:///tmp/resources".to_string(),
        visible: false,
        logging_on: true,
        event_mode_used: false,
        early_return_allowed: false,
        required_intermediate_variables: vec![],
        n_required_intermediate_variables: 0,
    };

    let mut payload = Vec::new();
    instantiate_msg.encode(&mut payload)?;

    let instance_response: InstanceMessage = fixture
        .query("fmi3InstantiateCoSimulation", &payload)
        .await?;

    let instance_index = instance_response.instance_index;
    println!("Instantiated instance: {}", instance_index);

    // 2. Enter Initialization Mode
    let init_msg = EnterInitializationModeMessage {
        instance_index,
        tolerance_defined: false,
        tolerance: 0.0,
        start_time: 0.0,
        stop_time_defined: true,
        stop_time: 10.0,
    };

    let mut payload = Vec::new();
    init_msg.encode(&mut payload)?;

    let init_response: StatusMessage = fixture
        .query("fmi3EnterInitializationMode", &payload)
        .await?;

    assert_eq!(init_response.status, Status::Ok as i32);
    println!("Entered initialization mode");

    // 3. Exit Initialization Mode
    let exit_msg = InstanceMessage { instance_index };

    let mut payload = Vec::new();
    exit_msg.encode(&mut payload)?;

    let exit_response: StatusMessage = fixture
        .query("fmi3ExitInitializationMode", &payload)
        .await?;

    assert_eq!(exit_response.status, Status::Ok as i32);
    println!("Exited initialization mode");

    // 4. Enter Step Mode
    let step_mode_msg = InstanceMessage { instance_index };

    let mut payload = Vec::new();
    step_mode_msg.encode(&mut payload)?;

    let step_mode_response: StatusMessage = fixture
        .query("fmi3EnterStepMode", &payload)
        .await?;

    assert_eq!(step_mode_response.status, Status::Ok as i32);
    println!("Entered step mode");

    // 5. Perform simulation steps
    for step in 0..5 {
        let time = step as f64 * 0.1;
        let do_step_msg = DoStepMessage {
            instance_index,
            current_communication_point: time,
            communication_step_size: 0.1,
            no_set_fmu_state_prior_to_current_point: true,
            event_handling_needed: false,
            terminate_simulation: false,
            early_return: false,
            last_successful_time: time,
        };

        let mut payload = Vec::new();
        do_step_msg.encode(&mut payload)?;

        let step_response: DoStepMessage = fixture.query("fmi3DoStep", &payload).await?;

        println!(
            "Step {}: time={}, last_successful_time={}",
            step, time, step_response.last_successful_time
        );

        assert!(!step_response.terminate_simulation);
    }

    // 6. Get variable values
    let get_msg = GetFloat64InputMessage {
        instance_index,
        value_references: vec![0, 1], // time and counter
        n_value_references: 2,
    };

    let mut payload = Vec::new();
    get_msg.encode(&mut payload)?;

    let get_response: GetFloat64OutputMessage =
        fixture.query("fmi3GetFloat64", &payload).await?;

    assert_eq!(get_response.status, Status::Ok as i32);
    assert_eq!(get_response.values.len(), 2);
    println!("Variable values: {:?}", get_response.values);

    // Counter should have incremented
    assert!(get_response.values[1] > 0.0);

    // 7. Terminate
    let terminate_msg = InstanceMessage { instance_index };

    let mut payload = Vec::new();
    terminate_msg.encode(&mut payload)?;

    let terminate_response: StatusMessage =
        fixture.query("fmi3Terminate", &payload).await?;

    assert_eq!(terminate_response.status, Status::Ok as i32);
    println!("Terminated instance");

    // 8. Free Instance
    let free_msg = InstanceMessage { instance_index };

    let mut payload = Vec::new();
    free_msg.encode(&mut payload)?;

    // fmi3FreeInstance doesn't return a value, but the query should succeed
    let _: StatusMessage = fixture.query("fmi3FreeInstance", &payload).await?;
    println!("Freed instance");

    Ok(())
}

#[tokio::test]
async fn test_set_and_get_variables() -> Result<()> {
    let fixture = TestFixture::new().await?;

    // Instantiate and initialize
    let instantiate_msg = InstantiateCoSimulationMessage {
        instance_name: "test_variables".to_string(),
        instantiation_token: "{12345678-1234-1234-1234-123456789012}".to_string(),
        resource_path: "file:///tmp/resources".to_string(),
        visible: false,
        logging_on: true,
        event_mode_used: false,
        early_return_allowed: false,
        required_intermediate_variables: vec![],
        n_required_intermediate_variables: 0,
    };

    let mut payload = Vec::new();
    instantiate_msg.encode(&mut payload)?;

    let instance_response: InstanceMessage = fixture
        .query("fmi3InstantiateCoSimulation", &payload)
        .await?;

    let instance_index = instance_response.instance_index;

    // Set variable value
    let set_msg = SetFloat64InputMessage {
        instance_index,
        value_references: vec![1], // counter
        n_value_references: 1,
        values: vec![42.0],
        n_values: 1,
    };

    let mut payload = Vec::new();
    set_msg.encode(&mut payload)?;

    let set_response: StatusMessage = fixture.query("fmi3SetFloat64", &payload).await?;
    assert_eq!(set_response.status, Status::Ok as i32);

    // Get the value back
    let get_msg = GetFloat64InputMessage {
        instance_index,
        value_references: vec![1], // counter
        n_value_references: 1,
    };

    let mut payload = Vec::new();
    get_msg.encode(&mut payload)?;

    let get_response: GetFloat64OutputMessage =
        fixture.query("fmi3GetFloat64", &payload).await?;

    assert_eq!(get_response.status, Status::Ok as i32);
    assert_eq!(get_response.values.len(), 1);
    assert_eq!(get_response.values[0], 42.0);

    println!("Successfully set and retrieved variable value: 42.0");

    Ok(())
}

#[tokio::test]
async fn test_multiple_instances() -> Result<()> {
    let fixture = TestFixture::new().await?;

    // Create two instances
    let mut instances = Vec::new();

    for i in 0..2 {
        let instantiate_msg = InstantiateCoSimulationMessage {
            instance_name: format!("test_instance_{}", i),
            instantiation_token: "{12345678-1234-1234-1234-123456789012}".to_string(),
            resource_path: "file:///tmp/resources".to_string(),
            visible: false,
            logging_on: true,
            event_mode_used: false,
            early_return_allowed: false,
            required_intermediate_variables: vec![],
            n_required_intermediate_variables: 0,
        };

        let mut payload = Vec::new();
        instantiate_msg.encode(&mut payload)?;

        let instance_response: InstanceMessage = fixture
            .query("fmi3InstantiateCoSimulation", &payload)
            .await?;

        instances.push(instance_response.instance_index);
        println!("Created instance {}: {}", i, instance_response.instance_index);
    }

    // Verify we got different instance indices
    assert_ne!(instances[0], instances[1]);

    println!("Successfully created multiple instances");

    Ok(())
}
