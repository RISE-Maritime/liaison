// Memory profiling tests for Liaison FMI
//
// This test suite creates multiple FMU instances and performs typical operations
// to measure memory usage at different stages and test for memory leaks.
//
// Run with:
//   cargo test --test memory_test --release -- --nocapture
//
// For memory profiling:
//   valgrind --tool=massif cargo test --test memory_test --release
//   heaptrack cargo test --test memory_test --release

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

// Memory tracking utilities
#[cfg(target_os = "linux")]
fn get_current_memory_usage() -> Option<usize> {
    use std::fs;

    // Read from /proc/self/status
    let status = fs::read_to_string("/proc/self/status").ok()?;

    // Find VmRSS (Resident Set Size - actual RAM used)
    for line in status.lines() {
        if line.starts_with("VmRSS:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                // Parse KB value
                return parts[1].parse::<usize>().ok().map(|kb| kb * 1024);
            }
        }
    }

    None
}

#[cfg(not(target_os = "linux"))]
fn get_current_memory_usage() -> Option<usize> {
    None
}

struct MemorySnapshot {
    stage: String,
    timestamp: Instant,
    memory_bytes: Option<usize>,
}

impl MemorySnapshot {
    fn new(stage: &str) -> Self {
        Self {
            stage: stage.to_string(),
            timestamp: Instant::now(),
            memory_bytes: get_current_memory_usage(),
        }
    }

    fn print(&self) {
        match self.memory_bytes {
            Some(bytes) => {
                let mb = bytes as f64 / (1024.0 * 1024.0);
                println!("[MEMORY] {} - {:.2} MB", self.stage, mb);
            }
            None => {
                println!("[MEMORY] {} - Unable to measure", self.stage);
            }
        }
    }
}

struct MemoryTracker {
    snapshots: Vec<MemorySnapshot>,
    start_time: Instant,
}

impl MemoryTracker {
    fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            start_time: Instant::now(),
        }
    }

    fn snapshot(&mut self, stage: &str) {
        let snapshot = MemorySnapshot::new(stage);
        snapshot.print();
        self.snapshots.push(snapshot);
    }

    fn print_summary(&self) {
        println!("\n========================================");
        println!("Memory Profile Summary");
        println!("========================================");
        println!("Total duration: {:?}", self.start_time.elapsed());
        println!("\nMemory usage by stage:");
        println!("{:<30} {:>15}", "Stage", "Memory (MB)");
        println!("{:-<45}", "");

        for snapshot in &self.snapshots {
            match snapshot.memory_bytes {
                Some(bytes) => {
                    let mb = bytes as f64 / (1024.0 * 1024.0);
                    println!("{:<30} {:>15.2}", snapshot.stage, mb);
                }
                None => {
                    println!("{:<30} {:>15}", snapshot.stage, "N/A");
                }
            }
        }

        // Calculate memory growth
        if self.snapshots.len() >= 2 {
            if let (Some(start), Some(end)) = (
                self.snapshots.first().and_then(|s| s.memory_bytes),
                self.snapshots.last().and_then(|s| s.memory_bytes),
            ) {
                let growth = end as i64 - start as i64;
                let growth_mb = growth as f64 / (1024.0 * 1024.0);
                println!("\n{:-<45}", "");
                println!("Total memory growth: {:.2} MB", growth_mb);

                if growth_mb > 10.0 {
                    println!("WARNING: Significant memory growth detected!");
                }
            }
        }

        println!("========================================\n");
    }
}

// Simulate FMU operations for memory testing
// In a real scenario, this would interact with actual FMU instances

struct MockFmuInstance {
    id: usize,
    // Simulate some memory allocation
    _data: Vec<f64>,
}

impl MockFmuInstance {
    fn new(id: usize, data_size: usize) -> Self {
        Self {
            id,
            _data: vec![0.0; data_size],
        }
    }

    fn do_step(&mut self, _current_time: f64, _step_size: f64) {
        // Simulate some work
        thread::sleep(Duration::from_millis(1));
    }

    fn set_values(&mut self, _values: &[f64]) {
        // Simulate setting values
    }

    fn get_values(&self) -> Vec<f64> {
        // Simulate getting values
        vec![0.0; 10]
    }
}

#[test]
fn test_memory_baseline() {
    let mut tracker = MemoryTracker::new();

    tracker.snapshot("Initial");

    // Small allocation to verify tracking works
    let _data: Vec<u8> = vec![0; 10 * 1024 * 1024]; // 10 MB
    tracker.snapshot("After 10MB allocation");

    drop(_data);
    tracker.snapshot("After deallocation");

    tracker.print_summary();
}

#[test]
fn test_single_instance_lifecycle() {
    let mut tracker = MemoryTracker::new();

    tracker.snapshot("Before instance creation");

    // Create a single FMU instance
    let mut instance = MockFmuInstance::new(1, 100_000);
    tracker.snapshot("After instance creation");

    // Initialize
    tracker.snapshot("After initialization");

    // Run simulation
    for i in 0..100 {
        instance.do_step(i as f64 * 0.01, 0.01);
    }
    tracker.snapshot("After 100 steps");

    // Access values
    for _ in 0..50 {
        let _values = instance.get_values();
        instance.set_values(&[1.0, 2.0, 3.0]);
    }
    tracker.snapshot("After value operations");

    // Cleanup
    drop(instance);
    tracker.snapshot("After instance cleanup");

    tracker.print_summary();
}

#[test]
fn test_multiple_instances() {
    let mut tracker = MemoryTracker::new();

    tracker.snapshot("Before creating instances");

    // Create multiple instances
    let mut instances = Vec::new();
    for i in 0..10 {
        instances.push(MockFmuInstance::new(i, 50_000));
    }
    tracker.snapshot("After creating 10 instances");

    // Run all instances
    for step in 0..50 {
        for instance in &mut instances {
            instance.do_step(step as f64 * 0.01, 0.01);
        }
    }
    tracker.snapshot("After running 50 steps");

    // Cleanup half
    instances.truncate(5);
    tracker.snapshot("After removing 5 instances");

    // Continue with remaining
    for step in 50..100 {
        for instance in &mut instances {
            instance.do_step(step as f64 * 0.01, 0.01);
        }
    }
    tracker.snapshot("After additional 50 steps");

    // Final cleanup
    instances.clear();
    tracker.snapshot("After all instances removed");

    tracker.print_summary();
}

#[test]
fn test_concurrent_instances() {
    let mut tracker = MemoryTracker::new();

    tracker.snapshot("Before concurrent test");

    let handles: Vec<_> = (0..5)
        .map(|i| {
            thread::spawn(move || {
                let mut instance = MockFmuInstance::new(i, 30_000);

                for step in 0..100 {
                    instance.do_step(step as f64 * 0.01, 0.01);
                }

                // Return to ensure instance is kept alive
                instance
            })
        })
        .collect();

    tracker.snapshot("After spawning 5 threads");

    // Wait for threads
    let _results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    tracker.snapshot("After threads completed");

    tracker.print_summary();
}

#[test]
fn memory_stress() {
    let mut tracker = MemoryTracker::new();

    tracker.snapshot("Start stress test");

    // Stress test: many rapid allocations and deallocations
    for iteration in 0..5 {
        let mut instances = Vec::new();

        // Create instances
        for i in 0..20 {
            instances.push(MockFmuInstance::new(i, 100_000));
        }

        tracker.snapshot(&format!("Iteration {} - created 20 instances", iteration));

        // Run a few steps
        for step in 0..10 {
            for instance in &mut instances {
                instance.do_step(step as f64 * 0.01, 0.01);
                let _values = instance.get_values();
            }
        }

        tracker.snapshot(&format!("Iteration {} - completed simulation", iteration));

        // Cleanup
        instances.clear();

        tracker.snapshot(&format!("Iteration {} - cleaned up", iteration));
    }

    tracker.snapshot("Stress test complete");

    tracker.print_summary();
}

#[test]
fn test_memory_leak_detection() {
    let mut tracker = MemoryTracker::new();

    tracker.snapshot("Initial state");

    // Baseline measurement
    let baseline_memory = get_current_memory_usage();

    // Perform 10 iterations of create/destroy cycles
    for i in 0..10 {
        let mut instances = Vec::new();

        for j in 0..10 {
            instances.push(MockFmuInstance::new(j, 50_000));
        }

        // Do some work
        for step in 0..20 {
            for instance in &mut instances {
                instance.do_step(step as f64 * 0.01, 0.01);
            }
        }

        // Cleanup
        instances.clear();

        if i % 2 == 1 {
            tracker.snapshot(&format!("After iteration {}", i));
        }
    }

    // Final measurement
    let final_memory = get_current_memory_usage();

    tracker.snapshot("Final state");

    // Check for leaks
    if let (Some(baseline), Some(final_mem)) = (baseline_memory, final_memory) {
        let growth = final_mem as i64 - baseline as i64;
        let growth_mb = growth as f64 / (1024.0 * 1024.0);

        println!("\n========================================");
        println!("Memory Leak Detection");
        println!("========================================");
        println!("Baseline memory: {:.2} MB", baseline as f64 / (1024.0 * 1024.0));
        println!("Final memory:    {:.2} MB", final_mem as f64 / (1024.0 * 1024.0));
        println!("Growth:          {:.2} MB", growth_mb);

        if growth_mb > 5.0 {
            println!("\nWARNING: Potential memory leak detected!");
            println!("Memory grew by {:.2} MB after repeated create/destroy cycles", growth_mb);
        } else {
            println!("\nOK: No significant memory leak detected");
        }
        println!("========================================\n");
    }

    tracker.print_summary();
}

#[test]
fn test_large_data_transfer() {
    let mut tracker = MemoryTracker::new();

    tracker.snapshot("Before large data test");

    let instance = MockFmuInstance::new(1, 1_000_000);
    tracker.snapshot("After creating large instance");

    // Simulate large data transfers
    for _ in 0..100 {
        let large_data: Vec<f64> = vec![1.0; 10_000];
        instance.set_values(&large_data);
        let _result = instance.get_values();
    }

    tracker.snapshot("After 100 large transfers");

    drop(instance);
    tracker.snapshot("After cleanup");

    tracker.print_summary();
}

// Performance metrics collection
struct PerformanceMetrics {
    operation: String,
    iterations: usize,
    total_duration: Duration,
    memory_start: Option<usize>,
    memory_end: Option<usize>,
}

impl PerformanceMetrics {
    fn new(operation: &str) -> Self {
        Self {
            operation: operation.to_string(),
            iterations: 0,
            total_duration: Duration::default(),
            memory_start: get_current_memory_usage(),
            memory_end: None,
        }
    }

    fn finish(&mut self) {
        self.memory_end = get_current_memory_usage();
    }

    fn print(&self) {
        println!("\n--- Performance Metrics: {} ---", self.operation);
        println!("Iterations: {}", self.iterations);
        println!("Total duration: {:?}", self.total_duration);

        if self.iterations > 0 {
            let avg_us = self.total_duration.as_micros() / self.iterations as u128;
            println!("Average time per iteration: {} μs", avg_us);
        }

        if let (Some(start), Some(end)) = (self.memory_start, self.memory_end) {
            let growth = end as i64 - start as i64;
            println!(
                "Memory change: {:.2} MB",
                growth as f64 / (1024.0 * 1024.0)
            );
        }
    }
}

#[test]
fn test_operation_performance() {
    // Test do_step performance
    let mut metrics = PerformanceMetrics::new("do_step");
    let mut instance = MockFmuInstance::new(1, 100_000);

    let start = Instant::now();
    for i in 0..1000 {
        instance.do_step(i as f64 * 0.01, 0.01);
        metrics.iterations += 1;
    }
    metrics.total_duration = start.elapsed();
    metrics.finish();
    metrics.print();

    // Test value access performance
    let mut metrics = PerformanceMetrics::new("get/set values");
    let start = Instant::now();
    for _ in 0..1000 {
        let _values = instance.get_values();
        instance.set_values(&[1.0, 2.0, 3.0]);
        metrics.iterations += 1;
    }
    metrics.total_duration = start.elapsed();
    metrics.finish();
    metrics.print();
}
