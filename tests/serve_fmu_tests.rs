// Integration tests for serving FMUs with the Liaison server
//
// These tests validate the `liaison serve` command functionality:
// 1. Starting a liaison server to serve an FMU
// 2. Verifying the server starts successfully and remains running
// 3. Testing graceful shutdown of the server
//
// Test strategy:
// - Use Command::spawn to run the liaison server in the background
// - Test with the BouncingBall.fmu reference FMU
// - Verify server process lifecycle management
// - Ensure proper cleanup even on test failure
//
// Limitations:
// - These tests primarily validate server startup and stability
// - Full FMI protocol interaction is tested in integration_test.rs
// - Network communication requires Zenoh to be available

use anyhow::{Context, Result};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

mod utils;

/// Helper function to get the path to a reference FMU
fn get_reference_fmu(name: &str) -> Result<PathBuf> {
    let workspace = utils::get_workspace_dir()?;
    let fmu_path = workspace.join("examples").join(name);

    if !fmu_path.exists() {
        anyhow::bail!(
            "Reference FMU not found at {}. Please ensure the file exists.",
            fmu_path.display()
        );
    }

    Ok(fmu_path)
}

/// Helper function to get the liaison binary path
fn get_liaison_binary() -> Result<PathBuf> {
    let workspace = utils::get_workspace_dir()?;

    // Prefer release build for performance
    let release_binary = workspace.join("target/release/liaison");
    if release_binary.exists() {
        return Ok(release_binary);
    }

    // Fallback to debug build
    let debug_binary = workspace.join("target/debug/liaison");
    if debug_binary.exists() {
        return Ok(debug_binary);
    }

    anyhow::bail!(
        "Liaison binary not found. Please run 'cargo build --release' or 'cargo build' first."
    );
}

/// Managed server handle that ensures proper cleanup
struct ServerProcess {
    child: Child,
    responder_id: String,
}

impl ServerProcess {
    /// Spawn a liaison server process
    fn spawn(fmu_path: &PathBuf, responder_id: &str, debug: bool) -> Result<Self> {
        let liaison_binary = get_liaison_binary()?;

        let mut cmd = Command::new(&liaison_binary);
        cmd.arg("serve")
            .arg(fmu_path)
            .arg(responder_id)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if debug {
            cmd.arg("--debug");
        }

        let child = cmd
            .spawn()
            .context("Failed to spawn liaison serve process")?;

        Ok(ServerProcess {
            child,
            responder_id: responder_id.to_string(),
        })
    }

    /// Check if the server process is still running
    fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Get the process ID
    fn pid(&self) -> u32 {
        self.child.id()
    }

    /// Wait for the server to start up (by checking if it stays alive)
    fn wait_for_startup(&mut self, timeout: Duration) -> Result<()> {
        let start = Instant::now();

        // Wait for initial startup
        thread::sleep(Duration::from_millis(500));

        // Verify the process is still running
        while start.elapsed() < timeout {
            if !self.is_running() {
                // Process died, try to get error output
                let output = self.child.wait_with_output()?;
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                anyhow::bail!(
                    "Server process died during startup.\nStdout: {}\nStderr: {}",
                    stdout,
                    stderr
                );
            }

            // If still running after a reasonable time, consider it started
            if start.elapsed() > Duration::from_secs(2) {
                return Ok(());
            }

            thread::sleep(Duration::from_millis(100));
        }

        Ok(())
    }

    /// Capture server output (stdout and stderr)
    fn capture_output(&mut self, lines: usize) -> Result<(Vec<String>, Vec<String>)> {
        let mut stdout_lines = Vec::new();
        let mut stderr_lines = Vec::new();

        if let Some(stdout) = self.child.stdout.take() {
            let reader = BufReader::new(stdout);
            stdout_lines = reader
                .lines()
                .take(lines)
                .filter_map(|line| line.ok())
                .collect();
        }

        if let Some(stderr) = self.child.stderr.take() {
            let reader = BufReader::new(stderr);
            stderr_lines = reader
                .lines()
                .take(lines)
                .filter_map(|line| line.ok())
                .collect();
        }

        Ok((stdout_lines, stderr_lines))
    }

    /// Stop the server gracefully
    fn stop(&mut self) -> Result<()> {
        if !self.is_running() {
            return Ok(());
        }

        // Try graceful termination first (SIGTERM on Unix)
        #[cfg(unix)]
        {
            unsafe {
                libc::kill(self.child.id() as i32, libc::SIGTERM);
            }
            thread::sleep(Duration::from_millis(500));
        }

        // If still running, force kill
        if self.is_running() {
            self.child.kill()?;
        }

        let _ = self.child.wait()?;
        Ok(())
    }
}

impl Drop for ServerProcess {
    fn drop(&mut self) {
        // Ensure the server is stopped when dropped (even on panic)
        let _ = self.stop();
    }
}

// ============================================================================
// Test Cases
// ============================================================================

#[test]
fn test_serve_bouncing_ball_basic() -> Result<()> {
    // Initialize logging for debugging
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init();

    let fmu_path = get_reference_fmu("BouncingBall.fmu")?;
    let responder_id = format!("test-serve-basic-{}", std::process::id());

    tracing::info!("Starting liaison server...");
    tracing::info!("  FMU: {}", fmu_path.display());
    tracing::info!("  Responder ID: {}", responder_id);

    let mut server = ServerProcess::spawn(&fmu_path, &responder_id, true)?;

    tracing::info!("  Server PID: {}", server.pid());

    // Wait for server to start
    server
        .wait_for_startup(Duration::from_secs(5))
        .context("Server failed to start within timeout")?;

    tracing::info!("Server started successfully");

    // Verify the server is still running after startup
    assert!(
        server.is_running(),
        "Server should be running after startup"
    );

    tracing::info!("Server is running and stable");

    // Let the server run for a bit to ensure stability
    thread::sleep(Duration::from_secs(1));

    assert!(
        server.is_running(),
        "Server should still be running after 1 second"
    );

    tracing::info!("Server remained stable for 1 second");

    // Clean up
    server.stop()?;

    tracing::info!("Server stopped successfully");

    Ok(())
}

#[test]
fn test_serve_process_lifecycle() -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init();

    let fmu_path = get_reference_fmu("BouncingBall.fmu")?;
    let responder_id = format!("test-lifecycle-{}", std::process::id());

    tracing::info!("Testing server process lifecycle...");

    let mut server = ServerProcess::spawn(&fmu_path, &responder_id, false)?;
    let pid = server.pid();

    tracing::info!("  Server started with PID: {}", pid);

    // Verify server is running
    assert!(server.is_running(), "Server should be running initially");

    // Wait for initialization
    server.wait_for_startup(Duration::from_secs(5))?;

    // Verify still running
    assert!(
        server.is_running(),
        "Server should be running after initialization"
    );

    // Stop the server
    tracing::info!("  Stopping server...");
    server.stop()?;

    // Verify server stopped
    assert!(
        !server.is_running(),
        "Server should not be running after stop"
    );

    tracing::info!("Server lifecycle test completed successfully");

    Ok(())
}

#[test]
fn test_serve_with_debug_output() -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init();

    let fmu_path = get_reference_fmu("BouncingBall.fmu")?;
    let responder_id = format!("test-debug-{}", std::process::id());

    tracing::info!("Testing server with debug output...");

    let mut server = ServerProcess::spawn(&fmu_path, &responder_id, true)?;

    // Wait for server to generate some output
    server.wait_for_startup(Duration::from_secs(3))?;

    tracing::info!("  Server started, checking output...");

    // Note: Reading output is tricky because it blocks if no data is available
    // We'll just verify the server started and runs successfully

    assert!(
        server.is_running(),
        "Server should be running with debug enabled"
    );

    tracing::info!("Server running successfully with debug output");

    // Clean up
    server.stop()?;

    Ok(())
}

#[test]
fn test_serve_multiple_responder_ids() -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init();

    let fmu_path = get_reference_fmu("BouncingBall.fmu")?;

    tracing::info!("Testing multiple servers with different responder IDs...");

    // Start first server
    let responder_id_1 = format!("test-multi-1-{}", std::process::id());
    let mut server1 = ServerProcess::spawn(&fmu_path, &responder_id_1, false)?;
    server1.wait_for_startup(Duration::from_secs(5))?;

    tracing::info!("  Server 1 started: {}", responder_id_1);

    // Start second server with different responder ID
    let responder_id_2 = format!("test-multi-2-{}", std::process::id());
    let mut server2 = ServerProcess::spawn(&fmu_path, &responder_id_2, false)?;
    server2.wait_for_startup(Duration::from_secs(5))?;

    tracing::info!("  Server 2 started: {}", responder_id_2);

    // Both servers should be running
    assert!(
        server1.is_running(),
        "Server 1 should still be running"
    );
    assert!(
        server2.is_running(),
        "Server 2 should be running"
    );

    tracing::info!("Both servers running successfully");

    // Verify they have different PIDs
    assert_ne!(
        server1.pid(),
        server2.pid(),
        "Servers should have different process IDs"
    );

    tracing::info!("  Server 1 PID: {}", server1.pid());
    tracing::info!("  Server 2 PID: {}", server2.pid());

    // Clean up (Drop will handle this, but being explicit)
    server1.stop()?;
    server2.stop()?;

    tracing::info!("Multiple server test completed successfully");

    Ok(())
}

#[test]
fn test_serve_stability_over_time() -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init();

    let fmu_path = get_reference_fmu("BouncingBall.fmu")?;
    let responder_id = format!("test-stability-{}", std::process::id());

    tracing::info!("Testing server stability over time...");

    let mut server = ServerProcess::spawn(&fmu_path, &responder_id, false)?;
    server.wait_for_startup(Duration::from_secs(5))?;

    tracing::info!("  Server started, testing stability...");

    // Check server is running at regular intervals
    let check_interval = Duration::from_millis(500);
    let total_duration = Duration::from_secs(5);
    let start = Instant::now();

    let mut checks = 0;
    while start.elapsed() < total_duration {
        assert!(
            server.is_running(),
            "Server should be running at check {}",
            checks
        );
        checks += 1;
        thread::sleep(check_interval);
    }

    tracing::info!("  Server remained stable through {} checks over {:?}", checks, total_duration);

    // Final verification
    assert!(
        server.is_running(),
        "Server should still be running after stability test"
    );

    // Clean up
    server.stop()?;

    tracing::info!("Stability test completed successfully");

    Ok(())
}

#[test]
fn test_serve_cleanup_on_drop() -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init();

    let fmu_path = get_reference_fmu("BouncingBall.fmu")?;
    let responder_id = format!("test-cleanup-{}", std::process::id());

    tracing::info!("Testing automatic cleanup on drop...");

    let pid = {
        let mut server = ServerProcess::spawn(&fmu_path, &responder_id, false)?;
        server.wait_for_startup(Duration::from_secs(5))?;

        let pid = server.pid();
        tracing::info!("  Server started with PID: {}", pid);

        assert!(server.is_running(), "Server should be running");

        pid
        // server goes out of scope here and should be cleaned up
    };

    // Give it a moment for cleanup to complete
    thread::sleep(Duration::from_millis(500));

    // Try to check if process is still alive (Unix-specific)
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;

        // Try to send signal 0 (doesn't actually send a signal, just checks if process exists)
        let result = unsafe { libc::kill(pid as i32, 0) };

        // If kill returns -1 and errno is ESRCH (3), the process doesn't exist
        if result == -1 {
            let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
            assert_eq!(
                errno, 3, // ESRCH - No such process
                "Process should not exist after drop (errno should be ESRCH)"
            );
            tracing::info!("  Process {} successfully cleaned up on drop", pid);
        } else {
            panic!(
                "Process {} still exists after drop, cleanup failed",
                pid
            );
        }
    }

    #[cfg(not(unix))]
    {
        tracing::info!("  Cleanup verification skipped on non-Unix platform");
    }

    tracing::info!("Cleanup test completed successfully");

    Ok(())
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_serve_with_nonexistent_fmu() -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init();

    let workspace = utils::get_workspace_dir()?;
    let nonexistent_fmu = workspace.join("NonExistent.fmu");
    let responder_id = format!("test-error-{}", std::process::id());

    tracing::info!("Testing server with nonexistent FMU...");

    // This should fail to spawn or die immediately
    let result = ServerProcess::spawn(&nonexistent_fmu, &responder_id, false);

    // The spawn might succeed but the process should die immediately
    if let Ok(mut server) = result {
        let startup_result = server.wait_for_startup(Duration::from_secs(2));

        assert!(
            startup_result.is_err(),
            "Server should fail to start with nonexistent FMU"
        );

        tracing::info!("  Server correctly failed to start with nonexistent FMU");
    } else {
        // Spawn failed, which is also acceptable
        tracing::info!("  Server correctly failed to spawn with nonexistent FMU");
    }

    Ok(())
}

#[test]
fn test_serve_graceful_shutdown() -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init();

    let fmu_path = get_reference_fmu("BouncingBall.fmu")?;
    let responder_id = format!("test-shutdown-{}", std::process::id());

    tracing::info!("Testing graceful server shutdown...");

    let mut server = ServerProcess::spawn(&fmu_path, &responder_id, false)?;
    server.wait_for_startup(Duration::from_secs(5))?;

    let pid = server.pid();
    tracing::info!("  Server running with PID: {}", pid);

    // Gracefully stop the server
    let start = Instant::now();
    server.stop()?;
    let shutdown_time = start.elapsed();

    tracing::info!("  Server shut down in {:?}", shutdown_time);

    // Verify shutdown was reasonably fast (should be < 2 seconds)
    assert!(
        shutdown_time < Duration::from_secs(2),
        "Shutdown should complete within 2 seconds"
    );

    // Verify server is not running
    assert!(
        !server.is_running(),
        "Server should not be running after shutdown"
    );

    tracing::info!("Graceful shutdown test completed successfully");

    Ok(())
}
