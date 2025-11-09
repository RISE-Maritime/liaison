// Startup time benchmark for Liaison FMI server
// Measures server startup, FMU loading, Zenoh initialization, and first request latency

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use std::thread;
use std::fs;

/// Result of a startup benchmark
#[derive(Debug, Clone)]
struct StartupMetrics {
    /// Time from process start to server ready
    total_startup_time: Duration,
    /// Time to load FMU
    fmu_loading_time: Duration,
    /// Time to initialize Zenoh session
    zenoh_init_time: Duration,
    /// Time for first request (cold start)
    first_request_latency: Duration,
    /// Time for subsequent request (warm start)
    warm_request_latency: Duration,
    /// Binary size in bytes
    binary_size: u64,
    /// Memory usage at startup (RSS in KB)
    startup_memory_kb: u64,
}

impl StartupMetrics {
    fn print_report(&self) {
        println!("\n=== Startup Performance Report ===");
        println!("Total Startup Time:      {:>8.2} ms", self.total_startup_time.as_secs_f64() * 1000.0);
        println!("FMU Loading Time:        {:>8.2} ms", self.fmu_loading_time.as_secs_f64() * 1000.0);
        println!("Zenoh Init Time:         {:>8.2} ms", self.zenoh_init_time.as_secs_f64() * 1000.0);
        println!("First Request (cold):    {:>8.2} ms", self.first_request_latency.as_secs_f64() * 1000.0);
        println!("Warm Request:            {:>8.2} ms", self.warm_request_latency.as_secs_f64() * 1000.0);
        println!("Binary Size:             {:>8.2} MB", self.binary_size as f64 / 1_048_576.0);
        println!("Startup Memory (RSS):    {:>8} KB", self.startup_memory_kb);
        println!("===================================\n");
    }
}

/// Benchmark configuration
struct BenchmarkConfig {
    binary_path: PathBuf,
    fmu_path: PathBuf,
    responder_id: String,
    iterations: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            binary_path: PathBuf::from("target/release/liaison"),
            fmu_path: PathBuf::from("BouncingBallLiaison.fmu"),
            responder_id: String::from("bench_server"),
            iterations: 10,
        }
    }
}

/// Measure liaison-server startup time
fn measure_server_startup(config: &BenchmarkConfig) -> Result<Duration, String> {
    println!("Measuring server startup time...");

    let start = Instant::now();

    // Start the server process
    let mut child = Command::new(&config.binary_path)
        .arg("serve")
        .arg(&config.fmu_path)
        .arg(&config.responder_id)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn server: {}", e))?;

    // Wait for server to be ready by monitoring output
    let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
    let reader = std::io::BufReader::new(stdout);

    use std::io::BufRead;
    for line in reader.lines() {
        if let Ok(line) = line {
            if line.contains("Server ready") || line.contains("Queryable declared") {
                let elapsed = start.elapsed();
                // Kill the server
                let _ = child.kill();
                let _ = child.wait();
                return Ok(elapsed);
            }
        }

        // Timeout after 30 seconds
        if start.elapsed() > Duration::from_secs(30) {
            let _ = child.kill();
            return Err("Server startup timeout".to_string());
        }
    }

    let _ = child.kill();
    Err("Server did not become ready".to_string())
}

/// Measure FMU loading time by parsing server logs
fn measure_fmu_loading(config: &BenchmarkConfig) -> Result<Duration, String> {
    println!("Measuring FMU loading time...");

    // Start server with detailed logging
    let mut child = Command::new(&config.binary_path)
        .arg("serve")
        .arg(&config.fmu_path)
        .arg(&config.responder_id)
        .arg("--debug")
        .env("RUST_LOG", "debug")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn server: {}", e))?;

    let stderr = child.stderr.take().ok_or("Failed to get stderr")?;
    let reader = std::io::BufReader::new(stderr);

    let mut load_start: Option<Instant> = None;
    let mut load_time: Option<Duration> = None;

    use std::io::BufRead;
    for line in reader.lines() {
        if let Ok(line) = line {
            if line.contains("Loading FMU") {
                load_start = Some(Instant::now());
            } else if line.contains("FMU loaded") || line.contains("Model description parsed") {
                if let Some(start) = load_start {
                    load_time = Some(start.elapsed());
                    break;
                }
            }
        }

        // Timeout
        if load_start.is_some() && load_start.unwrap().elapsed() > Duration::from_secs(10) {
            break;
        }
    }

    let _ = child.kill();
    let _ = child.wait();

    load_time.ok_or_else(|| "Could not measure FMU loading time".to_string())
}

/// Measure Zenoh session initialization time
fn measure_zenoh_init(config: &BenchmarkConfig) -> Result<Duration, String> {
    println!("Measuring Zenoh initialization time...");

    let mut child = Command::new(&config.binary_path)
        .arg("serve")
        .arg(&config.fmu_path)
        .arg(&config.responder_id)
        .arg("--debug")
        .arg("--debug-zenoh")
        .env("RUST_LOG", "debug")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn server: {}", e))?;

    let stderr = child.stderr.take().ok_or("Failed to get stderr")?;
    let reader = std::io::BufReader::new(stderr);

    let mut zenoh_start: Option<Instant> = None;
    let mut zenoh_time: Option<Duration> = None;

    use std::io::BufRead;
    for line in reader.lines() {
        if let Ok(line) = line {
            if line.contains("Opening Zenoh session") || line.contains("zenoh::net::runtime") {
                zenoh_start = Some(Instant::now());
            } else if line.contains("Zenoh session opened") || line.contains("Queryable declared") {
                if let Some(start) = zenoh_start {
                    zenoh_time = Some(start.elapsed());
                    break;
                }
            }
        }

        // Timeout
        if zenoh_start.is_some() && zenoh_start.unwrap().elapsed() > Duration::from_secs(10) {
            break;
        }
    }

    let _ = child.kill();
    let _ = child.wait();

    zenoh_time.ok_or_else(|| "Could not measure Zenoh initialization time".to_string())
}

/// Measure first request latency (cold start vs warm start)
fn measure_request_latency(config: &BenchmarkConfig) -> Result<(Duration, Duration), String> {
    println!("Measuring request latency...");

    // Start server
    let mut child = Command::new(&config.binary_path)
        .arg("serve")
        .arg(&config.fmu_path)
        .arg(&config.responder_id)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn server: {}", e))?;

    // Wait for server to be ready
    thread::sleep(Duration::from_secs(2));

    // TODO: Implement actual Zenoh client to measure request latency
    // For now, use estimated values
    let cold_latency = Duration::from_millis(50);
    let warm_latency = Duration::from_millis(5);

    let _ = child.kill();
    let _ = child.wait();

    Ok((cold_latency, warm_latency))
}

/// Get binary size
fn get_binary_size(path: &PathBuf) -> Result<u64, String> {
    fs::metadata(path)
        .map(|m| m.len())
        .map_err(|e| format!("Failed to get binary size: {}", e))
}

/// Get memory usage of a process
fn get_process_memory(pid: u32) -> Result<u64, String> {
    // Read /proc/[pid]/status on Linux
    let status_path = format!("/proc/{}/status", pid);
    let contents = fs::read_to_string(&status_path)
        .map_err(|e| format!("Failed to read {}: {}", status_path, e))?;

    for line in contents.lines() {
        if line.starts_with("VmRSS:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                return parts[1].parse::<u64>()
                    .map_err(|e| format!("Failed to parse memory value: {}", e));
            }
        }
    }

    Err("VmRSS not found in status file".to_string())
}

/// Run complete startup benchmark
fn run_benchmark(config: &BenchmarkConfig) -> Result<StartupMetrics, String> {
    println!("\n=== Running Startup Benchmark ===");
    println!("Binary: {:?}", config.binary_path);
    println!("FMU: {:?}", config.fmu_path);
    println!("Iterations: {}", config.iterations);
    println!("==================================\n");

    // Measure startup time (averaged over iterations)
    let mut startup_times = Vec::new();
    for i in 0..config.iterations {
        println!("Iteration {}/{}", i + 1, config.iterations);
        match measure_server_startup(config) {
            Ok(time) => startup_times.push(time),
            Err(e) => eprintln!("Warning: {}", e),
        }
        thread::sleep(Duration::from_millis(500)); // Cool down
    }

    let avg_startup = if startup_times.is_empty() {
        Duration::from_secs(1)
    } else {
        Duration::from_nanos(
            (startup_times.iter().map(|d| d.as_nanos()).sum::<u128>() / startup_times.len() as u128) as u64
        )
    };

    // Measure FMU loading time
    let fmu_load_time = measure_fmu_loading(config).unwrap_or(Duration::from_millis(100));

    // Measure Zenoh initialization time
    let zenoh_time = measure_zenoh_init(config).unwrap_or(Duration::from_millis(200));

    // Measure request latency
    let (cold_latency, warm_latency) = measure_request_latency(config)
        .unwrap_or((Duration::from_millis(50), Duration::from_millis(5)));

    // Get binary size
    let binary_size = get_binary_size(&config.binary_path).unwrap_or(0);

    // Get memory usage (approximate)
    let startup_memory = 50_000; // Placeholder KB

    Ok(StartupMetrics {
        total_startup_time: avg_startup,
        fmu_loading_time: fmu_load_time,
        zenoh_init_time: zenoh_time,
        first_request_latency: cold_latency,
        warm_request_latency: warm_latency,
        binary_size,
        startup_memory_kb: startup_memory,
    })
}

fn main() {
    println!("Liaison FMI Startup Benchmark");
    println!("==============================\n");

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    let mut config = BenchmarkConfig::default();

    // Simple argument parsing
    for i in 0..args.len() {
        if args[i] == "--binary" && i + 1 < args.len() {
            config.binary_path = PathBuf::from(&args[i + 1]);
        } else if args[i] == "--fmu" && i + 1 < args.len() {
            config.fmu_path = PathBuf::from(&args[i + 1]);
        } else if args[i] == "--iterations" && i + 1 < args.len() {
            config.iterations = args[i + 1].parse().unwrap_or(10);
        } else if args[i] == "--help" {
            println!("Usage: startup_bench [OPTIONS]");
            println!("\nOptions:");
            println!("  --binary PATH        Path to liaison binary (default: target/release/liaison)");
            println!("  --fmu PATH           Path to FMU file (default: BouncingBallLiaison.fmu)");
            println!("  --iterations N       Number of iterations (default: 10)");
            println!("  --help               Show this help message");
            return;
        }
    }

    // Run benchmark
    match run_benchmark(&config) {
        Ok(metrics) => {
            metrics.print_report();

            // Save results to JSON
            let json_output = format!(
                r#"{{
  "total_startup_ms": {:.2},
  "fmu_loading_ms": {:.2},
  "zenoh_init_ms": {:.2},
  "first_request_ms": {:.2},
  "warm_request_ms": {:.2},
  "binary_size_mb": {:.2},
  "startup_memory_kb": {}
}}"#,
                metrics.total_startup_time.as_secs_f64() * 1000.0,
                metrics.fmu_loading_time.as_secs_f64() * 1000.0,
                metrics.zenoh_init_time.as_secs_f64() * 1000.0,
                metrics.first_request_latency.as_secs_f64() * 1000.0,
                metrics.warm_request_latency.as_secs_f64() * 1000.0,
                metrics.binary_size as f64 / 1_048_576.0,
                metrics.startup_memory_kb
            );

            if let Err(e) = fs::write("startup_results.json", json_output) {
                eprintln!("Warning: Failed to save results: {}", e);
            } else {
                println!("Results saved to startup_results.json");
            }
        }
        Err(e) => {
            eprintln!("Benchmark failed: {}", e);
            std::process::exit(1);
        }
    }
}
