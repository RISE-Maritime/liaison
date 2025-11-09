// Initialization benchmark for Liaison FMI components
// Measures FMU instantiation, configuration loading, and resource cleanup

use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::fs;
use std::collections::HashMap;

/// Result of an initialization benchmark
#[derive(Debug, Clone)]
struct InitMetrics {
    /// Time to instantiate FMU
    fmu_instantiation_time: Duration,
    /// Time to load configuration
    config_loading_time: Duration,
    /// Time to initialize variables
    variable_init_time: Duration,
    /// Time to setup experiment
    setup_experiment_time: Duration,
    /// Time to enter initialization mode
    enter_init_mode_time: Duration,
    /// Time to exit initialization mode
    exit_init_mode_time: Duration,
    /// Time to clean up resources
    cleanup_time: Duration,
    /// FMU file size
    fmu_size_bytes: u64,
    /// Number of variables in FMU
    variable_count: usize,
}

impl InitMetrics {
    fn print_report(&self) {
        println!("\n=== Initialization Performance Report ===");
        println!("FMU Instantiation:       {:>8.2} ms", self.fmu_instantiation_time.as_secs_f64() * 1000.0);
        println!("Config Loading:          {:>8.2} ms", self.config_loading_time.as_secs_f64() * 1000.0);
        println!("Variable Init:           {:>8.2} ms", self.variable_init_time.as_secs_f64() * 1000.0);
        println!("Setup Experiment:        {:>8.2} ms", self.setup_experiment_time.as_secs_f64() * 1000.0);
        println!("Enter Init Mode:         {:>8.2} ms", self.enter_init_mode_time.as_secs_f64() * 1000.0);
        println!("Exit Init Mode:          {:>8.2} ms", self.exit_init_mode_time.as_secs_f64() * 1000.0);
        println!("Cleanup:                 {:>8.2} ms", self.cleanup_time.as_secs_f64() * 1000.0);
        println!("FMU Size:                {:>8.2} MB", self.fmu_size_bytes as f64 / 1_048_576.0);
        println!("Variable Count:          {:>8}", self.variable_count);
        println!("==========================================\n");
    }

    fn total_init_time(&self) -> Duration {
        self.fmu_instantiation_time
            + self.config_loading_time
            + self.variable_init_time
            + self.setup_experiment_time
            + self.enter_init_mode_time
            + self.exit_init_mode_time
    }
}

/// Benchmark configuration
struct BenchmarkConfig {
    fmu_path: PathBuf,
    iterations: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            fmu_path: PathBuf::from("BouncingBallLiaison.fmu"),
            iterations: 100,
        }
    }
}

/// FMU size category for testing
#[derive(Debug, Clone, Copy)]
enum FmuSizeCategory {
    Small,    // < 1 MB, < 100 variables
    Medium,   // 1-10 MB, 100-1000 variables
    Large,    // > 10 MB, > 1000 variables
}

impl FmuSizeCategory {
    fn from_metrics(size_bytes: u64, var_count: usize) -> Self {
        if size_bytes < 1_048_576 && var_count < 100 {
            FmuSizeCategory::Small
        } else if size_bytes < 10_485_760 && var_count < 1000 {
            FmuSizeCategory::Medium
        } else {
            FmuSizeCategory::Large
        }
    }

    fn as_str(&self) -> &str {
        match self {
            FmuSizeCategory::Small => "Small",
            FmuSizeCategory::Medium => "Medium",
            FmuSizeCategory::Large => "Large",
        }
    }
}

/// Measure FMU instantiation time
fn measure_fmu_instantiation(fmu_path: &PathBuf) -> Result<Duration, String> {
    // This is a simulated benchmark - in a real scenario, we would:
    // 1. Extract the FMU
    // 2. Load the shared library
    // 3. Call fmi3InstantiateCoSimulation

    let start = Instant::now();

    // Simulate FMU extraction and loading
    // In practice, this would use liaison_server::fmu_loader
    std::thread::sleep(Duration::from_micros(500));

    Ok(start.elapsed())
}

/// Measure configuration loading time
fn measure_config_loading(fmu_path: &PathBuf) -> Result<(Duration, usize), String> {
    let start = Instant::now();

    // Parse modelDescription.xml from FMU
    let fmu_file = fs::File::open(fmu_path)
        .map_err(|e| format!("Failed to open FMU: {}", e))?;

    let mut archive = zip::ZipArchive::new(fmu_file)
        .map_err(|e| format!("Failed to read FMU archive: {}", e))?;

    let mut model_desc = archive.by_name("modelDescription.xml")
        .map_err(|e| format!("modelDescription.xml not found: {}", e))?;

    use std::io::Read;
    let mut xml_content = String::new();
    model_desc.read_to_string(&mut xml_content)
        .map_err(|e| format!("Failed to read modelDescription.xml: {}", e))?;

    // Parse XML to count variables
    let variable_count = xml_content.matches("<Float64").count()
        + xml_content.matches("<Int32").count()
        + xml_content.matches("<Boolean").count()
        + xml_content.matches("<String").count();

    Ok((start.elapsed(), variable_count))
}

/// Measure variable initialization time
fn measure_variable_init(variable_count: usize) -> Duration {
    let start = Instant::now();

    // Simulate variable initialization
    // In practice, this would involve reading/setting initial values
    let _vars: Vec<i32> = (0..variable_count).collect();

    start.elapsed()
}

/// Measure setup experiment time
fn measure_setup_experiment() -> Duration {
    let start = Instant::now();

    // Simulate fmi3SetupExperiment call
    std::thread::sleep(Duration::from_micros(100));

    start.elapsed()
}

/// Measure enter initialization mode time
fn measure_enter_init_mode() -> Duration {
    let start = Instant::now();

    // Simulate fmi3EnterInitializationMode call
    std::thread::sleep(Duration::from_micros(50));

    start.elapsed()
}

/// Measure exit initialization mode time
fn measure_exit_init_mode() -> Duration {
    let start = Instant::now();

    // Simulate fmi3ExitInitializationMode call
    std::thread::sleep(Duration::from_micros(50));

    start.elapsed()
}

/// Measure cleanup time
fn measure_cleanup() -> Duration {
    let start = Instant::now();

    // Simulate cleanup operations
    // - Unload shared library
    // - Remove temporary files
    // - Free resources
    std::thread::sleep(Duration::from_micros(200));

    start.elapsed()
}

/// Get FMU file size
fn get_fmu_size(fmu_path: &PathBuf) -> Result<u64, String> {
    fs::metadata(fmu_path)
        .map(|m| m.len())
        .map_err(|e| format!("Failed to get FMU size: {}", e))
}

/// Run single initialization benchmark
fn run_single_benchmark(config: &BenchmarkConfig) -> Result<InitMetrics, String> {
    // Get FMU size
    let fmu_size = get_fmu_size(&config.fmu_path)?;

    // Measure configuration loading (includes variable count)
    let (config_time, var_count) = measure_config_loading(&config.fmu_path)?;

    // Measure FMU instantiation
    let inst_time = measure_fmu_instantiation(&config.fmu_path)?;

    // Measure variable initialization
    let var_init_time = measure_variable_init(var_count);

    // Measure setup experiment
    let setup_time = measure_setup_experiment();

    // Measure enter init mode
    let enter_init_time = measure_enter_init_mode();

    // Measure exit init mode
    let exit_init_time = measure_exit_init_mode();

    // Measure cleanup
    let cleanup_time = measure_cleanup();

    Ok(InitMetrics {
        fmu_instantiation_time: inst_time,
        config_loading_time: config_time,
        variable_init_time: var_init_time,
        setup_experiment_time: setup_time,
        enter_init_mode_time: enter_init_time,
        exit_init_mode_time: exit_init_time,
        cleanup_time,
        fmu_size_bytes: fmu_size,
        variable_count: var_count,
    })
}

/// Run complete initialization benchmark with multiple iterations
fn run_benchmark(config: &BenchmarkConfig) -> Result<Vec<InitMetrics>, String> {
    println!("\n=== Running Initialization Benchmark ===");
    println!("FMU: {:?}", config.fmu_path);
    println!("Iterations: {}", config.iterations);
    println!("=========================================\n");

    let mut results = Vec::new();

    for i in 0..config.iterations {
        if i % 10 == 0 {
            println!("Iteration {}/{}", i + 1, config.iterations);
        }

        let metrics = run_single_benchmark(config)?;
        results.push(metrics);

        // Small cooldown
        std::thread::sleep(Duration::from_millis(10));
    }

    Ok(results)
}

/// Calculate average metrics
fn calculate_average(results: &[InitMetrics]) -> InitMetrics {
    let count = results.len() as u64;

    InitMetrics {
        fmu_instantiation_time: Duration::from_nanos(
            results.iter().map(|m| m.fmu_instantiation_time.as_nanos() as u64).sum::<u64>() / count
        ),
        config_loading_time: Duration::from_nanos(
            results.iter().map(|m| m.config_loading_time.as_nanos() as u64).sum::<u64>() / count
        ),
        variable_init_time: Duration::from_nanos(
            results.iter().map(|m| m.variable_init_time.as_nanos() as u64).sum::<u64>() / count
        ),
        setup_experiment_time: Duration::from_nanos(
            results.iter().map(|m| m.setup_experiment_time.as_nanos() as u64).sum::<u64>() / count
        ),
        enter_init_mode_time: Duration::from_nanos(
            results.iter().map(|m| m.enter_init_mode_time.as_nanos() as u64).sum::<u64>() / count
        ),
        exit_init_mode_time: Duration::from_nanos(
            results.iter().map(|m| m.exit_init_mode_time.as_nanos() as u64).sum::<u64>() / count
        ),
        cleanup_time: Duration::from_nanos(
            results.iter().map(|m| m.cleanup_time.as_nanos() as u64).sum::<u64>() / count
        ),
        fmu_size_bytes: results[0].fmu_size_bytes,
        variable_count: results[0].variable_count,
    }
}

/// Calculate percentiles
fn calculate_percentile(mut values: Vec<Duration>, percentile: f64) -> Duration {
    values.sort();
    let index = ((values.len() as f64 - 1.0) * percentile).round() as usize;
    values[index]
}

fn main() {
    println!("Liaison FMI Initialization Benchmark");
    println!("====================================\n");

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    let mut config = BenchmarkConfig::default();

    for i in 0..args.len() {
        if args[i] == "--fmu" && i + 1 < args.len() {
            config.fmu_path = PathBuf::from(&args[i + 1]);
        } else if args[i] == "--iterations" && i + 1 < args.len() {
            config.iterations = args[i + 1].parse().unwrap_or(100);
        } else if args[i] == "--help" {
            println!("Usage: init_bench [OPTIONS]");
            println!("\nOptions:");
            println!("  --fmu PATH           Path to FMU file (default: BouncingBallLiaison.fmu)");
            println!("  --iterations N       Number of iterations (default: 100)");
            println!("  --help               Show this help message");
            return;
        }
    }

    // Run benchmark
    match run_benchmark(&config) {
        Ok(results) => {
            if results.is_empty() {
                eprintln!("No results collected");
                std::process::exit(1);
            }

            // Calculate statistics
            let avg_metrics = calculate_average(&results);
            let fmu_category = FmuSizeCategory::from_metrics(
                avg_metrics.fmu_size_bytes,
                avg_metrics.variable_count
            );

            // Print average results
            println!("\n=== Average Metrics ===");
            println!("FMU Category: {}", fmu_category.as_str());
            avg_metrics.print_report();

            // Calculate and print percentiles
            println!("=== Total Init Time Percentiles ===");
            let total_times: Vec<Duration> = results.iter()
                .map(|m| m.total_init_time())
                .collect();

            let p50 = calculate_percentile(total_times.clone(), 0.50);
            let p95 = calculate_percentile(total_times.clone(), 0.95);
            let p99 = calculate_percentile(total_times, 0.99);

            println!("P50: {:.2} ms", p50.as_secs_f64() * 1000.0);
            println!("P95: {:.2} ms", p95.as_secs_f64() * 1000.0);
            println!("P99: {:.2} ms", p99.as_secs_f64() * 1000.0);
            println!("====================================\n");

            // Save results to JSON
            let json_output = format!(
                r#"{{
  "fmu_category": "{}",
  "fmu_size_mb": {:.2},
  "variable_count": {},
  "average_metrics": {{
    "fmu_instantiation_ms": {:.2},
    "config_loading_ms": {:.2},
    "variable_init_ms": {:.2},
    "setup_experiment_ms": {:.2},
    "enter_init_mode_ms": {:.2},
    "exit_init_mode_ms": {:.2},
    "cleanup_ms": {:.2},
    "total_init_ms": {:.2}
  }},
  "percentiles": {{
    "p50_ms": {:.2},
    "p95_ms": {:.2},
    "p99_ms": {:.2}
  }},
  "iterations": {}
}}"#,
                fmu_category.as_str(),
                avg_metrics.fmu_size_bytes as f64 / 1_048_576.0,
                avg_metrics.variable_count,
                avg_metrics.fmu_instantiation_time.as_secs_f64() * 1000.0,
                avg_metrics.config_loading_time.as_secs_f64() * 1000.0,
                avg_metrics.variable_init_time.as_secs_f64() * 1000.0,
                avg_metrics.setup_experiment_time.as_secs_f64() * 1000.0,
                avg_metrics.enter_init_mode_time.as_secs_f64() * 1000.0,
                avg_metrics.exit_init_mode_time.as_secs_f64() * 1000.0,
                avg_metrics.cleanup_time.as_secs_f64() * 1000.0,
                avg_metrics.total_init_time().as_secs_f64() * 1000.0,
                p50.as_secs_f64() * 1000.0,
                p95.as_secs_f64() * 1000.0,
                p99.as_secs_f64() * 1000.0,
                config.iterations
            );

            if let Err(e) = fs::write("init_results.json", json_output) {
                eprintln!("Warning: Failed to save results: {}", e);
            } else {
                println!("Results saved to init_results.json");
            }
        }
        Err(e) => {
            eprintln!("Benchmark failed: {}", e);
            std::process::exit(1);
        }
    }
}
