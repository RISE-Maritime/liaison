//! Network Latency Benchmarks for Liaison FMI
//!
//! This benchmark measures round-trip time for FMI function calls over Zenoh.
//! It tests various FMI operations with different payload sizes and collects
//! detailed latency statistics.
//!
//! # Benchmark Categories
//!
//! 1. **Lifecycle Operations**: Instantiation, initialization, termination
//! 2. **Data Operations**: getValue/setValue with varying payload sizes
//! 3. **Simulation Operations**: doStep with different step sizes
//!
//! # Metrics Collected
//!
//! - Minimum latency
//! - Maximum latency
//! - Mean latency
//! - Median (p50)
//! - 95th percentile (p95)
//! - 99th percentile (p99)
//! - Standard deviation
//!
//! # Usage
//!
//! ```bash
//! # Compile the benchmark
//! cargo build --release --bin latency_test
//!
//! # Run the benchmark
//! ./target/release/latency_test <fmu_path> <responder_id>
//!
//! # Example
//! ./target/release/latency_test ./BouncingBallLiaison.fmu test_responder
//!
//! # Run with custom iterations
//! ./target/release/latency_test ./BouncingBallLiaison.fmu test_responder --iterations 1000
//!
//! # Run specific test category
//! ./target/release/latency_test ./BouncingBallLiaison.fmu test_responder --test lifecycle
//! ./target/release/latency_test ./BouncingBallLiaison.fmu test_responder --test data
//! ./target/release/latency_test ./BouncingBallLiaison.fmu test_responder --test simulation
//! ```
//!
//! # Output
//!
//! Results are printed to stdout in both human-readable and CSV formats.
//! JSON output is also available for automated processing.

use anyhow::{Context, Result};
use prost::Message;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use zenoh::Wait;

// Include generated protobuf types
mod proto {
    include!(concat!(env!("OUT_DIR"), "/proto.rs"));
}

/// Statistics for latency measurements
#[derive(Debug, Clone)]
struct LatencyStats {
    min: Duration,
    max: Duration,
    mean: Duration,
    median: Duration,
    p95: Duration,
    p99: Duration,
    std_dev: Duration,
    samples: Vec<Duration>,
}

impl LatencyStats {
    fn from_samples(mut samples: Vec<Duration>) -> Self {
        samples.sort();
        let count = samples.len();

        let min = *samples.first().unwrap();
        let max = *samples.last().unwrap();

        let sum: Duration = samples.iter().sum();
        let mean = sum / count as u32;

        let median = samples[count / 2];
        let p95 = samples[(count as f64 * 0.95) as usize];
        let p99 = samples[(count as f64 * 0.99) as usize];

        // Calculate standard deviation
        let variance: f64 = samples
            .iter()
            .map(|&d| {
                let diff = d.as_secs_f64() - mean.as_secs_f64();
                diff * diff
            })
            .sum::<f64>()
            / count as f64;
        let std_dev = Duration::from_secs_f64(variance.sqrt());

        LatencyStats {
            min,
            max,
            mean,
            median,
            p95,
            p99,
            std_dev,
            samples,
        }
    }

    fn print_summary(&self, test_name: &str) {
        println!("\n{}", "=".repeat(80));
        println!("Latency Statistics: {}", test_name);
        println!("{}", "=".repeat(80));
        println!("Samples:        {}", self.samples.len());
        println!("Min:            {:>10.3} ms", self.min.as_secs_f64() * 1000.0);
        println!("Max:            {:>10.3} ms", self.max.as_secs_f64() * 1000.0);
        println!("Mean:           {:>10.3} ms", self.mean.as_secs_f64() * 1000.0);
        println!("Median (p50):   {:>10.3} ms", self.median.as_secs_f64() * 1000.0);
        println!("p95:            {:>10.3} ms", self.p95.as_secs_f64() * 1000.0);
        println!("p99:            {:>10.3} ms", self.p99.as_secs_f64() * 1000.0);
        println!("Std Dev:        {:>10.3} ms", self.std_dev.as_secs_f64() * 1000.0);
        println!("{}", "=".repeat(80));
    }

    fn to_csv_row(&self, test_name: &str) -> String {
        format!(
            "{},{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3}",
            test_name,
            self.samples.len(),
            self.min.as_secs_f64() * 1000.0,
            self.max.as_secs_f64() * 1000.0,
            self.mean.as_secs_f64() * 1000.0,
            self.median.as_secs_f64() * 1000.0,
            self.p95.as_secs_f64() * 1000.0,
            self.p99.as_secs_f64() * 1000.0,
            self.std_dev.as_secs_f64() * 1000.0
        )
    }

    fn to_json(&self, test_name: &str) -> String {
        format!(
            r#"{{
  "test_name": "{}",
  "samples": {},
  "min_ms": {:.3},
  "max_ms": {:.3},
  "mean_ms": {:.3},
  "median_ms": {:.3},
  "p95_ms": {:.3},
  "p99_ms": {:.3},
  "std_dev_ms": {:.3}
}}"#,
            test_name,
            self.samples.len(),
            self.min.as_secs_f64() * 1000.0,
            self.max.as_secs_f64() * 1000.0,
            self.mean.as_secs_f64() * 1000.0,
            self.median.as_secs_f64() * 1000.0,
            self.p95.as_secs_f64() * 1000.0,
            self.p99.as_secs_f64() * 1000.0,
            self.std_dev.as_secs_f64() * 1000.0
        )
    }
}

/// Benchmark configuration
struct BenchmarkConfig {
    fmu_path: PathBuf,
    responder_id: String,
    iterations: usize,
    warmup_iterations: usize,
    test_category: Option<String>,
    output_format: OutputFormat,
}

#[derive(Debug, Clone, Copy)]
enum OutputFormat {
    Human,
    Csv,
    Json,
}

/// Latency benchmark runner
struct LatencyBenchmark {
    config: BenchmarkConfig,
    session: zenoh::Session,
    results: HashMap<String, LatencyStats>,
}

impl LatencyBenchmark {
    async fn new(config: BenchmarkConfig) -> Result<Self> {
        let session = zenoh::open(zenoh::Config::default())
            .await
            .wait()
            .context("Failed to open Zenoh session")?;

        Ok(LatencyBenchmark {
            config,
            session,
            results: HashMap::new(),
        })
    }

    /// Run all benchmarks based on configuration
    async fn run_all(&mut self) -> Result<()> {
        println!("Starting Liaison FMI Network Latency Benchmarks");
        println!("FMU: {}", self.config.fmu_path.display());
        println!("Responder ID: {}", self.config.responder_id);
        println!("Iterations: {}", self.config.iterations);
        println!("Warmup iterations: {}", self.config.warmup_iterations);
        println!();

        match &self.config.test_category {
            Some(category) => match category.as_str() {
                "lifecycle" => self.run_lifecycle_benchmarks().await?,
                "data" => self.run_data_benchmarks().await?,
                "simulation" => self.run_simulation_benchmarks().await?,
                _ => anyhow::bail!("Unknown test category: {}", category),
            },
            None => {
                self.run_lifecycle_benchmarks().await?;
                self.run_data_benchmarks().await?;
                self.run_simulation_benchmarks().await?;
            }
        }

        self.print_results();
        Ok(())
    }

    /// Benchmark lifecycle operations (instantiation, initialization, etc.)
    async fn run_lifecycle_benchmarks(&mut self) -> Result<()> {
        println!("Running lifecycle operation benchmarks...");

        // Test InstantiateCoSimulation
        let stats = self
            .benchmark_operation(
                "InstantiateCoSimulation",
                || {
                    let msg = proto::Fmi3InstantiateCoSimulationMessage {
                        instance_name: "test_instance".to_string(),
                        instantiation_token: "{guid}".to_string(),
                        resource_path: "".to_string(),
                        visible: false,
                        logging_on: false,
                        event_mode_used: false,
                        early_return_allowed: false,
                        required_intermediate_variables: vec![],
                        n_required_intermediate_variables: 0,
                    };
                    (
                        format!("{}/fmi3InstantiateCoSimulation", self.config.responder_id),
                        msg.encode_to_vec(),
                    )
                },
            )
            .await?;
        self.results.insert("InstantiateCoSimulation".to_string(), stats);

        // Small delay between tests
        thread::sleep(Duration::from_millis(100));

        // Test EnterInitializationMode
        let stats = self
            .benchmark_operation(
                "EnterInitializationMode",
                || {
                    let msg = proto::Fmi3EnterInitializationModeMessage {
                        instance_index: 0,
                        tolerance_defined: false,
                        tolerance: 0.0,
                        start_time: 0.0,
                        stop_time_defined: false,
                        stop_time: 0.0,
                    };
                    (
                        format!("{}/fmi3EnterInitializationMode", self.config.responder_id),
                        msg.encode_to_vec(),
                    )
                },
            )
            .await?;
        self.results.insert("EnterInitializationMode".to_string(), stats);

        thread::sleep(Duration::from_millis(100));

        // Test ExitInitializationMode
        let stats = self
            .benchmark_operation(
                "ExitInitializationMode",
                || {
                    let msg = proto::Fmi3InstanceMessage { instance_index: 0 };
                    (
                        format!("{}/fmi3ExitInitializationMode", self.config.responder_id),
                        msg.encode_to_vec(),
                    )
                },
            )
            .await?;
        self.results.insert("ExitInitializationMode".to_string(), stats);

        Ok(())
    }

    /// Benchmark data operations with varying payload sizes
    async fn run_data_benchmarks(&mut self) -> Result<()> {
        println!("Running data operation benchmarks...");

        // Test GetFloat64 with different payload sizes
        for num_values in [1, 10, 100, 1000] {
            let test_name = format!("GetFloat64_{}_values", num_values);
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
                )
                .await?;
            self.results.insert(test_name, stats);
            thread::sleep(Duration::from_millis(100));
        }

        // Test SetFloat64 with different payload sizes
        for num_values in [1, 10, 100, 1000] {
            let test_name = format!("SetFloat64_{}_values", num_values);
            let stats = self
                .benchmark_operation(
                    &test_name,
                    || {
                        let msg = proto::Fmi3SetFloat64InputMessage {
                            instance_index: 0,
                            value_references: (0..num_values as i32).collect(),
                            n_value_references: num_values as i32,
                            values: vec![1.0; num_values],
                            n_values: num_values as i32,
                        };
                        (
                            format!("{}/fmi3SetFloat64", self.config.responder_id),
                            msg.encode_to_vec(),
                        )
                    },
                )
                .await?;
            self.results.insert(test_name, stats);
            thread::sleep(Duration::from_millis(100));
        }

        // Test different data types (single value)
        let data_types = vec![
            ("Int32", "fmi3GetInt32"),
            ("Boolean", "fmi3GetBoolean"),
            ("String", "fmi3GetString"),
        ];

        for (type_name, operation) in data_types {
            let test_name = format!("Get{}", type_name);
            let stats = self
                .benchmark_operation(
                    &test_name,
                    || {
                        let msg = proto::Fmi3GetInt32InputMessage {
                            instance_index: 0,
                            value_references: vec![0],
                            n_value_references: 1,
                        };
                        (
                            format!("{}/{}", self.config.responder_id, operation),
                            msg.encode_to_vec(),
                        )
                    },
                )
                .await?;
            self.results.insert(test_name, stats);
            thread::sleep(Duration::from_millis(100));
        }

        Ok(())
    }

    /// Benchmark simulation operations (doStep)
    async fn run_simulation_benchmarks(&mut self) -> Result<()> {
        println!("Running simulation operation benchmarks...");

        // Test DoStep with different step sizes
        for step_size in [0.001, 0.01, 0.1, 1.0] {
            let test_name = format!("DoStep_{}s", step_size);
            let stats = self
                .benchmark_operation(
                    &test_name,
                    || {
                        let msg = proto::Fmi3DoStepMessage {
                            instance_index: 0,
                            current_communication_point: 0.0,
                            communication_step_size: step_size,
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
                )
                .await?;
            self.results.insert(test_name, stats);
            thread::sleep(Duration::from_millis(100));
        }

        Ok(())
    }

    /// Generic benchmark for any operation
    async fn benchmark_operation<F>(&self, name: &str, op_factory: F) -> Result<LatencyStats>
    where
        F: Fn() -> (String, Vec<u8>),
    {
        println!("  Benchmarking: {}", name);

        let mut samples = Vec::with_capacity(self.config.iterations + self.config.warmup_iterations);

        // Warmup phase
        for _ in 0..self.config.warmup_iterations {
            let (key_expr, payload) = op_factory();
            let _ = self.single_request(&key_expr, payload).await;
        }

        // Measurement phase
        for i in 0..self.config.iterations {
            let (key_expr, payload) = op_factory();

            match self.single_request(&key_expr, payload).await {
                Ok(latency) => samples.push(latency),
                Err(e) => {
                    eprintln!("    Warning: Request {} failed: {}", i, e);
                    continue;
                }
            }

            // Progress indicator for long benchmarks
            if (i + 1) % 100 == 0 {
                print!("    Progress: {}/{}\r", i + 1, self.config.iterations);
            }
        }

        if !samples.is_empty() {
            println!("    Completed: {} samples collected", samples.len());
            Ok(LatencyStats::from_samples(samples))
        } else {
            anyhow::bail!("No successful samples collected for {}", name);
        }
    }

    /// Execute a single request and measure round-trip time
    async fn single_request(&self, key_expr: &str, payload: Vec<u8>) -> Result<Duration> {
        let start = Instant::now();

        let _reply = self
            .session
            .get(key_expr)
            .payload(payload)
            .await
            .wait()
            .context("Failed to send Zenoh request")?
            .recv_async()
            .await
            .context("Failed to receive Zenoh reply")?;

        let elapsed = start.elapsed();
        Ok(elapsed)
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
                println!("test_name,samples,min_ms,max_ms,mean_ms,median_ms,p95_ms,p99_ms,std_dev_ms");
                for (name, stats) in &self.results {
                    println!("{}", stats.to_csv_row(name));
                }
            }
            OutputFormat::Json => {
                println!("[");
                let mut first = true;
                for (name, stats) in &self.results {
                    if !first {
                        println!(",");
                    }
                    println!("{}", stats.to_json(name));
                    first = false;
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
        eprintln!("  --iterations <N>        Number of iterations (default: 100)");
        eprintln!("  --warmup <N>            Number of warmup iterations (default: 10)");
        eprintln!("  --test <category>       Test category: lifecycle, data, or simulation");
        eprintln!("  --format <format>       Output format: human, csv, or json (default: human)");
        eprintln!("  --no-server             Don't start a server (use existing one)");
        std::process::exit(1);
    }

    let fmu_path = PathBuf::from(&args[1]);
    let responder_id = args[2].clone();

    let mut iterations = 100;
    let mut warmup = 10;
    let mut test_category = None;
    let mut output_format = OutputFormat::Human;
    let mut start_server_flag = true;

    // Parse optional arguments
    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--iterations" => {
                iterations = args[i + 1].parse()?;
                i += 2;
            }
            "--warmup" => {
                warmup = args[i + 1].parse()?;
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
        iterations,
        warmup_iterations: warmup,
        test_category,
        output_format,
    };

    // Run benchmarks
    let result = async {
        let mut benchmark = LatencyBenchmark::new(config).await?;
        benchmark.run_all().await
    }
    .await;

    // Clean up server
    if let Some(mut server) = server.take() {
        let _ = server.kill();
    }

    result
}
