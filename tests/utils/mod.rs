// Test utilities for integration tests

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

/// Configuration for test server
pub struct ServerConfig {
    pub fmu_path: PathBuf,
    pub responder_id: String,
    pub zenoh_config: Option<PathBuf>,
    pub debug: bool,
}

/// Handle for a running liaison server instance
pub struct ServerHandle {
    process: Child,
    pub responder_id: String,
}

impl ServerHandle {
    /// Start a liaison server with the given configuration
    pub fn start(config: ServerConfig) -> Result<Self> {
        let server_binary = find_server_binary()?;

        let mut cmd = Command::new(server_binary);
        cmd.arg("serve")
            .arg(&config.fmu_path)
            .arg(&config.responder_id)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(zenoh_config) = &config.zenoh_config {
            cmd.arg("--zenoh-config").arg(zenoh_config);
        }

        if config.debug {
            cmd.arg("--debug");
        }

        let process = cmd
            .spawn()
            .context("Failed to start liaison server")?;

        // Give the server time to start up and initialize Zenoh
        thread::sleep(Duration::from_secs(2));

        Ok(ServerHandle {
            process,
            responder_id: config.responder_id,
        })
    }

    /// Check if the server process is still running
    pub fn is_running(&mut self) -> bool {
        matches!(self.process.try_wait(), Ok(None))
    }

    /// Stop the server gracefully
    pub fn stop(&mut self) -> Result<()> {
        // Try to terminate gracefully first
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            unsafe {
                libc::kill(self.process.id() as i32, libc::SIGTERM);
            }
            thread::sleep(Duration::from_millis(500));
        }

        #[cfg(windows)]
        {
            // On Windows, just kill the process
            self.process.kill()?;
        }

        // If still running, force kill
        if self.is_running() {
            self.process.kill()?;
        }

        self.process.wait()?;
        Ok(())
    }
}

impl Drop for ServerHandle {
    fn drop(&mut self) {
        // Ensure the server is stopped when the handle is dropped
        let _ = self.stop();
    }
}

/// Find the liaison server binary
fn find_server_binary() -> Result<PathBuf> {
    // Try to find the binary in the target directory
    let workspace_dir = get_workspace_dir()?;

    // Check debug build first
    let debug_binary = workspace_dir.join("target/debug/liaison");
    if debug_binary.exists() {
        return Ok(debug_binary);
    }

    // Check release build
    let release_binary = workspace_dir.join("target/release/liaison");
    if release_binary.exists() {
        return Ok(release_binary);
    }

    anyhow::bail!("Could not find liaison server binary. Please run 'cargo build' first.");
}

/// Get the workspace directory
pub fn get_workspace_dir() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;

    // Look for Cargo.toml with [workspace]
    let mut dir = current_dir.as_path();
    loop {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = std::fs::read_to_string(&cargo_toml)?;
            if content.contains("[workspace]") {
                return Ok(dir.to_path_buf());
            }
        }

        match dir.parent() {
            Some(parent) => dir = parent,
            None => anyhow::bail!("Could not find workspace root"),
        }
    }
}

/// Get the path to test fixtures
pub fn get_fixtures_dir() -> Result<PathBuf> {
    Ok(get_workspace_dir()?.join("tests/fixtures"))
}

/// Build the mock FMU if it doesn't exist
pub fn ensure_mock_fmu_built() -> Result<PathBuf> {
    let fixtures_dir = get_fixtures_dir()?;
    let fmu_path = fixtures_dir.join("build/MockFMU.fmu");

    // Check if FMU already exists
    if fmu_path.exists() {
        return Ok(fmu_path);
    }

    // Build the mock FMU
    println!("Building mock FMU...");
    let build_script = fixtures_dir.join("build_mock_fmu.sh");

    let output = Command::new("bash")
        .arg(&build_script)
        .current_dir(&fixtures_dir)
        .output()
        .context("Failed to run build script")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to build mock FMU: {}", stderr);
    }

    if !fmu_path.exists() {
        anyhow::bail!("Mock FMU was not created at expected path");
    }

    println!("Mock FMU built successfully");
    Ok(fmu_path)
}

/// Wait for a condition with timeout
pub fn wait_for<F>(mut condition: F, timeout: Duration) -> Result<()>
where
    F: FnMut() -> bool,
{
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if condition() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(100));
    }
    anyhow::bail!("Timeout waiting for condition")
}

/// Helper to create a Zenoh session for testing
pub async fn create_test_zenoh_session() -> Result<zenoh::Session> {
    let config = zenoh::Config::default();
    let session = zenoh::open(config)
        .await
        .context("Failed to create Zenoh session")?;
    Ok(session)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_dir() {
        let workspace = get_workspace_dir().unwrap();
        assert!(workspace.join("Cargo.toml").exists());
    }

    #[test]
    fn test_fixtures_dir() {
        let fixtures = get_fixtures_dir().unwrap();
        assert!(fixtures.exists());
    }
}
