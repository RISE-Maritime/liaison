//! Network Throughput Benchmarks for Liaison FMI
//!
//! This benchmark measures requests per second and data transfer rates for
//! FMI operations over Zenoh. It tests concurrent clients, sustained load
//! scenarios, and overall system capacity.
//!
//! # Benchmark Categories
//!
//! 1. **Request Rate**: Measure requests per second (RPS) for various operations
//! 2. **Concurrent Clients**: Test multiple simultaneous clients
//! 3. **Data Transfer**: Measure bytes per second for different payload sizes
//! 4. **Sustained Load**: Test system behavior under continuous load
//!
//! # Metrics Collected
//!
//! - Requests per second (RPS)
//! - Data throughput (MB/s)
//! - Success rate (%)
//! - Error rate
//! - Average response time
//! - CPU and memory usage (if available)
//!
//! # Usage
//!
//! ```bash
//! # Compile the benchmark
//! cargo build --release --bin throughput_test
//!
//! # Run the benchmark
//! ./target/release/throughput_test <fmu_path> <responder_id>
//!
//! # Example
//! ./target/release/throughput_test ./BouncingBallLiaison.fmu test_responder
//!
//! # Run with custom duration
//! ./target/release/throughput_test ./BouncingBallLiaison.fmu test_responder --duration 60
//!
//! # Run with multiple concurrent clients
//! ./target/release/throughput_test ./BouncingBallLiaison.fmu test_responder --clients 10
//!
//! # Run specific test category
//! ./target/release/throughput_test ./BouncingBallLiaison.fmu test_responder --test rps
//! ./target/release/throughput_test ./BouncingBallLiaison.fmu test_responder --test concurrent
//! ./target/release/throughput_test ./BouncingBallLiaison.fmu test_responder --test transfer
//! ```

use anyhow::{Context, Result};
use prost::Message;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tokio::task::JoinSet;
use zenoh::Wait;

// Include generated protobuf types
mod proto {
    include!(concat!(env!("OUT_DIR"), "/proto.rs"));
}

/// Throughput statistics
#[derive(Debug, Clone)]
struct ThroughputStats {
    duration: Duration,
    total_requests: usize,
    successful_requests: usize,
    failed_requests: usize,
    total_bytes: u64,
    requests_per_second: f64,
    throughput_mbps: f64,
    success_rate: f64,
    avg_response_time_ms: f64,
}

impl ThroughputStats {
    fn calculate(
        duration: Duration,
        total_requests: usize,
        successful_requests: usize,
        total_bytes: u64,
        total_response_time: Duration,
    ) -> Self {
        let duration_secs = duration.as_secs_f64();
        let requests_per_second = successful_requests as f64 / duration_secs;
        let throughput_mbps = (total_bytes as f64 / duration_secs) / (1024.0 * 1024.0);
        let success_rate = if total_requests > 0 {
            (successful_requests as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };
        let avg_response_time_ms = if successful_requests > 0 {
            total_response_time.as_secs_f64() * 1000.0 / successful_requests as f64
        } else {
            0.0
        };

        ThroughputStats {
            duration,
            total_requests,
            successful_requests,
            failed_requests: total_requests - successful_requests,
            total_bytes,
            requests_per_second,
            throughput_mbps,
            success_rate,
            avg_response_time_ms,
        }
    }

    fn print_summary(&self, test_name: &str) {
        println!("\n{}", "=".repeat(80));
        println!("Throughput Statistics: {}", test_name);
        println!("{}", "=".repeat(80));
        println!("Duration:           {:.2} s", self.duration.as_secs_f64());
        println!("Total Requests:     {}", self.total_requests);
        println!("Successful:         {}", self.successful_requests);
        println!("Failed:             {}", self.failed_requests);
        println!("Success Rate:       {:.2}%", self.success_rate);
        println!("Requests/sec:       {:.2}", self.requests_per_second);
        println!("Throughput:         {:.3} MB/s", self.throughput_mbps);
        println!("Avg Response:       {:.3} ms", self.avg_response_time_ms);
        println!("Total Data:         {:.2} MB", self.total_bytes as f64 / (1024.0 * 1024.0));
        println!("{}", "=".repeat(80));
    }

    fn to_csv_row(&self, test_name: &str) -> String {
        format!(
            "{},{:.2},{},{},{},{:.2},{:.2},{:.3},{:.3},{:.2}",
            test_name,
            self.duration.as_secs_f64(),
            self.total_requests,
            self.successful_requests,
            self.failed_requests,
            self.success_rate,
            self.requests_per_second,
            self.throughput_mbps,
            self.avg_response_time_ms,
            self.total_bytes as f64 / (1024.0 * 1024.0)
        )
    }

    fn to_json(&self, test_name: &str) -> String {
        format!(
            r#"{{
  "test_name": "{}",
  "duration_s": {:.2},
  "total_requests": {},
  "successful_requests": {},
  "failed_requests": {},
  "success_rate": {:.2},
  "requests_per_second": {:.2},
  "throughput_mbps": {:.3},
  "avg_response_time_ms": {:.3},
  "total_data_mb": {:.2}
}}"#,
            test_name,
            self.duration.as_secs_f64(),
            self.total_requests,
            self.successful_requests,
            self.failed_requests,
            self.success_rate,
            self.requests_per_second,
            self.throughput_mbps,
            self.avg_response_time_ms,
            self.total_bytes as f64 / (1024.0 * 1024.0)
        )
    }
}

/// Benchmark configuration
struct BenchmarkConfig {
    fmu_path: PathBuf,
    responder_id: String,
    duration_secs: u64,
    num_clients: usize,
    test_category: Option<String>,
    output_format: OutputFormat,
}

#[derive(Debug, Clone, Copy)]
enum OutputFormat {
    Human,
    Csv,
    Json,
}

/// Throughput benchmark runner
struct ThroughputBenchmark {
    config: BenchmarkConfig,
    session: zenoh::Session,
    results: Vec<(String, ThroughputStats)>,
}

impl ThroughputBenchmark {
    async fn new(config: BenchmarkConfig) -> Result<Self> {
        let session = zenoh::open(zenoh::Config::default())
            .await
            .wait()
            .context("Failed to open Zenoh session")?;

        Ok(ThroughputBenchmark {
            config,
            session,
            results: Vec::new(),
        })
    }

    /// Run all benchmarks based on configuration
    async fn run_all(&mut self) -> Result<()> {
        println!("Starting Liaison FMI Network Throughput Benchmarks");
        println!("FMU: {}", self.config.fmu_path.display());
        println!("Responder ID: {}", self.config.responder_id);
        println!("Duration: {} seconds", self.config.duration_secs);
        println!("Concurrent clients: {}", self.config.num_clients);
        println!();

        match &self.config.test_category {
            Some(category) => match category.as_str() {
                "rps" => self.run_rps_benchmarks().await?,
                "concurrent" => self.run_concurrent_benchmarks().await?,
                "transfer" => self.run_transfer_benchmarks().await?,
                "sustained" => self.run_sustained_load_benchmark().await?,
                _ => anyhow::bail!("Unknown test category: {}", category),
            },
            None => {
                self.run_rps_benchmarks().await?;
                self.run_concurrent_benchmarks().await?;
                self.run_transfer_benchmarks().await?;
                self.run_sustained_load_benchmark().await?;
            }
        }

        self.print_results();
        Ok(())
    }

    /// Benchmark requests per second for different operations
    async fn run_rps_benchmarks(&mut self) -> Result<()> {
        println!("Running requests per second benchmarks...");

        // Test DoStep RPS
        let stats = self
            .benchmark_operation(
                "DoStep_RPS",
                || {
                    let msg = proto::Fmi3DoStepMessage {
                        instance_index: 0,
                        current_communication_point: 0.0,
                        communication_step_size: 0.01,
                        no_set_fmu_state_prior_to_current_point: true,
                        event_handling_needed: false,
                        terminate_simulation: false,
                        early_return: false,
                        last_successful_time: 0.0,
                    };
                    (
                        format!("{}/fmi3DoStep", self.config.responder_id),
                        msg.encode_to_vec(),
                    )
                },
                1,
            )
            .await?;
        self.results.push(("DoStep_RPS".to_string(), stats));

        thread::sleep(Duration::from_millis(500));

        // Test GetFloat64 RPS
        let stats = self
            .benchmark_operation(
                "GetFloat64_RPS",
                || {
                    let msg = proto::Fmi3GetFloat64InputMessage {
                        instance_index: 0,
                        value_references: vec![0],
                        n_value_references: 1,
                    };
                    (
                        format!("{}/fmi3GetFloat64", self.config.responder_id),
                        msg.encode_to_vec(),
                    )
                },
                1,
            )
            .await?;
        self.results.push(("GetFloat64_RPS".to_string(), stats));

        thread::sleep(Duration::from_millis(500));

        // Test SetFloat64 RPS
        let stats = self
            .benchmark_operation(
                "SetFloat64_RPS",
                || {
                    let msg = proto::Fmi3SetFloat64InputMessage {
                        instance_index: 0,
                        value_references: vec![0],
                        n_value_references: 1,
                        values: vec![1.0],
                        n_values: 1,
                    };
                    (
                        format!("{}/fmi3SetFloat64", self.config.responder_id),
                        msg.encode_to_vec(),
                    )
                },
                1,
            )
            .await?;
        self.results.push(("SetFloat64_RPS".to_string(), stats));

        Ok(())
    }

    /// Benchmark concurrent client performance
    async fn run_concurrent_benchmarks(&mut self) -> Result<()> {
        println!("Running concurrent client benchmarks...");

        for num_clients in [1, 2, 4, 8, 16] {
            if num_clients > self.config.num_clients {
                break;
            }

            let test_name = format!("DoStep_{}clients", num_clients);
            let stats = self
                .benchmark_operation(
                    &test_name,
                    || {
                        let msg = proto::Fmi3DoStepMessage {
                            instance_index: 0,
                            current_communication_point: 0.0,
                            communication_step_size: 0.01,
                            no_set_fmu_state_prior_to_current_point: true,
                            event_handling_needed: false,
                            terminate_simulation: false,
                            early_return: false,
                            last_successful_time: 0.0,
                        };
                        (
                            format!("{}/fmi3DoStep", self.config.responder_id),
                            msg.encode_to_vec(),
                        )
                    },
                    num_clients,
                )
                .await?;
            self.results.push((test_name, stats));
            thread::sleep(Duration::from_millis(500));
        }

        Ok(())
    }

    /// Benchmark data transfer rates with different payload sizes
    async fn run_transfer_benchmarks(&mut self) -> Result<()> {
        println!("Running data transfer benchmarks...");

        for num_values in [10, 100, 1000, 10000] {
            let test_name = format!("Transfer_{}values", num_values);
            let stats = self
                .benchmark_operation(
                    &test_name,
                    || {
                        let msg = proto::Fmi3GetFloat64InputMessage {
                            instance_index: 0,
                            value_references: (0..num_values as i32).collect(),
                            n_value_references: num_values as i32,
                        };
                        (
                            format!("{}/fmi3GetFloat64", self.config.responder_id),
                            msg.encode_to_vec(),
                        )
                    },
                    1,
                )
                .await?;
            self.results.push((test_name, stats));
            thread::sleep(Duration::from_millis(500));
        }

        Ok(())
    }

    /// Run a sustained load test
    async fn run_sustained_load_benchmark(&mut self) -> Result<()> {
        println!("Running sustained load benchmark...");

        let stats = self
            .benchmark_operation(
                "Sustained_Mixed_Load",
                || {
                    // Mix of operations to simulate realistic usage
                    let ops = [
                        ("fmi3DoStep", 60),  // 60% DoStep
                        ("fmi3GetFloat64", 30), // 30% GetFloat64
                        ("fmi3SetFloat64", 10), // 10% SetFloat64
                    ];
                    let choice = (Instant::now().elapsed().as_nanos() % 100) as usize;
                    let cumulative = [60, 90, 100];
                    let selected = cumulative
                        .iter()
                        .position(|&c| choice < c)
                        .unwrap_or(0);

                    match ops[selected].0 {
                        "fmi3DoStep" => {
                            let msg = proto::Fmi3DoStepMessage {
                                instance_index: 0,
                                current_communication_point: 0.0,
                                communication_step_size: 0.01,
                                no_set_fmu_state_prior_to_current_point: true,
                                event_handling_needed: false,
                                terminate_simulation: false,
                                early_return: false,
                                last_successful_time: 0.0,
                            };
                            (
                                format!("{}/fmi3DoStep", self.config.responder_id),
                                msg.encode_to_vec(),
                            )
                        }
                        "fmi3GetFloat64" => {
                            let msg = proto::Fmi3GetFloat64InputMessage {
                                instance_index: 0,
                                value_references: vec![0],
                                n_value_references: 1,
                            };
                            (
                                format!("{}/fmi3GetFloat64", self.config.responder_id),
                                msg.encode_to_vec(),
                            )
                        }
                        _ => {
                            let msg = proto::Fmi3SetFloat64InputMessage {
                                instance_index: 0,
                                value_references: vec![0],
                                n_value_references: 1,
                                values: vec![1.0],
                                n_values: 1,
                            };
                            (
                                format!("{}/fmi3SetFloat64", self.config.responder_id),
                                msg.encode_to_vec(),
                            )
                        }
                    }
                },
                self.config.num_clients,
            )
            .await?;
        self.results.push(("Sustained_Mixed_Load".to_string(), stats));

        Ok(())
    }

    /// Generic benchmark for any operation with configurable concurrency
    async fn benchmark_operation<F>(
        &self,
        name: &str,
        op_factory: F,
        num_clients: usize,
    ) -> Result<ThroughputStats>
    where
        F: Fn() -> (String, Vec<u8>) + Send + Sync + 'static,
    {
        println!("  Benchmarking: {} (clients: {})", name, num_clients);

        let total_requests = Arc::new(AtomicUsize::new(0));
        let successful_requests = Arc::new(AtomicUsize::new(0));
        let total_bytes = Arc::new(AtomicU64::new(0));
        let total_response_time = Arc::new(std::sync::Mutex::new(Duration::ZERO));

        let duration = Duration::from_secs(self.config.duration_secs);
        let start = Instant::now();
        let stop_time = start + duration;

        let op_factory = Arc::new(op_factory);
        let mut tasks = JoinSet::new();

        // Spawn worker tasks
        for _ in 0..num_clients {
            let session = self.session.clone();
            let total_requests = total_requests.clone();
            let successful_requests = successful_requests.clone();
            let total_bytes = total_bytes.clone();
            let total_response_time = total_response_time.clone();
            let op_factory = op_factory.clone();

            tasks.spawn(async move {
                while Instant::now() < stop_time {
                    let (key_expr, payload) = op_factory();
                    let payload_size = payload.len() as u64;

                    total_requests.fetch_add(1, Ordering::Relaxed);

                    let req_start = Instant::now();
                    match session.get(&key_expr).payload(payload).await {
                        Ok(receiver) => {
                            match receiver.wait().recv_async().await {
                                Ok(_reply) => {
                                    let elapsed = req_start.elapsed();
                                    successful_requests.fetch_add(1, Ordering::Relaxed);
                                    total_bytes.fetch_add(payload_size, Ordering::Relaxed);
                                    if let Ok(mut rt) = total_response_time.lock() {
                                        *rt += elapsed;
                                    }
                                }
                                Err(_) => {
                                    // Failed to receive reply
                                }
                            }
                        }
                        Err(_) => {
                            // Failed to send request
                        }
                    }
                }
            });
        }

        // Wait for all tasks to complete
        while tasks.join_next().await.is_some() {}

        let actual_duration = start.elapsed();
        let total_req = total_requests.load(Ordering::Relaxed);
        let successful_req = successful_requests.load(Ordering::Relaxed);
        let bytes = total_bytes.load(Ordering::Relaxed);
        let response_time = *total_response_time.lock().unwrap();

        println!(
            "    Completed: {} requests in {:.2}s ({:.2} req/s)",
            successful_req,
            actual_duration.as_secs_f64(),
            successful_req as f64 / actual_duration.as_secs_f64()
        );

        Ok(ThroughputStats::calculate(
            actual_duration,
            total_req,
            successful_req,
            bytes,
            response_time,
        ))
    }

    /// Print benchmark results in the configured format
    fn print_results(&self) {
        match self.config.output_format {
            OutputFormat::Human => {
                for (name, stats) in &self.results {
                    stats.print_summary(name);
                }
            }
            OutputFormat::Csv => {
                println!("test_name,duration_s,total_requests,successful_requests,failed_requests,success_rate,requests_per_second,throughput_mbps,avg_response_time_ms,total_data_mb");
                for (name, stats) in &self.results {
                    println!("{}", stats.to_csv_row(name));
                }
            }
            OutputFormat::Json => {
                println!("[");
                for (i, (name, stats)) in self.results.iter().enumerate() {
                    if i > 0 {
                        println!(",");
                    }
                    println!("{}", stats.to_json(name));
                }
                println!("]");
            }
        }
    }
}

/// Start a liaison server in the background for testing
fn start_server(fmu_path: &PathBuf, responder_id: &str) -> Result<std::process::Child> {
    let liaison_binary = find_liaison_binary()?;

    let child = Command::new(liaison_binary)
        .arg("serve")
        .arg(fmu_path)
        .arg(responder_id)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to start liaison server")?;

    // Give server time to start
    thread::sleep(Duration::from_secs(2));

    Ok(child)
}

/// Find the liaison binary
fn find_liaison_binary() -> Result<PathBuf> {
    let workspace = std::env::current_dir()?;

    // Try release build first
    let release = workspace.join("target/release/liaison");
    if release.exists() {
        return Ok(release);
    }

    // Fall back to debug build
    let debug = workspace.join("target/debug/liaison");
    if debug.exists() {
        return Ok(debug);
    }

    anyhow::bail!("Liaison binary not found. Run 'cargo build --release' first.");
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <fmu_path> <responder_id> [options]", args[0]);
        eprintln!();
        eprintln!("Options:");
        eprintln!("  --duration <seconds>    Test duration in seconds (default: 10)");
        eprintln!("  --clients <N>           Maximum concurrent clients (default: 16)");
        eprintln!("  --test <category>       Test category: rps, concurrent, transfer, or sustained");
        eprintln!("  --format <format>       Output format: human, csv, or json (default: human)");
        eprintln!("  --no-server             Don't start a server (use existing one)");
        std::process::exit(1);
    }

    let fmu_path = PathBuf::from(&args[1]);
    let responder_id = args[2].clone();

    let mut duration_secs = 10;
    let mut num_clients = 16;
    let mut test_category = None;
    let mut output_format = OutputFormat::Human;
    let mut start_server_flag = true;

    // Parse optional arguments
    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--duration" => {
                duration_secs = args[i + 1].parse()?;
                i += 2;
            }
            "--clients" => {
                num_clients = args[i + 1].parse()?;
                i += 2;
            }
            "--test" => {
                test_category = Some(args[i + 1].clone());
                i += 2;
            }
            "--format" => {
                output_format = match args[i + 1].as_str() {
                    "human" => OutputFormat::Human,
                    "csv" => OutputFormat::Csv,
                    "json" => OutputFormat::Json,
                    _ => anyhow::bail!("Invalid format: {}", args[i + 1]),
                };
                i += 2;
            }
            "--no-server" => {
                start_server_flag = false;
                i += 1;
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                std::process::exit(1);
            }
        }
    }

    // Start server if requested
    let mut server = if start_server_flag {
        Some(start_server(&fmu_path, &responder_id)?)
    } else {
        None
    };

    let config = BenchmarkConfig {
        fmu_path,
        responder_id,
        duration_secs,
        num_clients,
        test_category,
        output_format,
    };

    // Run benchmarks
    let result = async {
        let mut benchmark = ThroughputBenchmark::new(config).await?;
        benchmark.run_all().await
    }
    .await;

    // Clean up server
    if let Some(mut server) = server.take() {
        let _ = server.kill();
    }

    result
}
