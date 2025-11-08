// Liaison Server
// Serves FMU instances over Zenoh network

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;

mod proto;
mod server;
mod fmu_creator;
mod utils;
mod fmu_loader;
mod instance_manager;
mod callbacks;
mod queryable_handlers;

#[derive(Parser)]
#[command(name = "liaison")]
#[command(about = "FMI 3.0 network liaison server and FMU creator", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Serve an FMU over the network
    Serve {
        /// Path to the FMU file
        fmu_path: PathBuf,

        /// Responder ID for Zenoh network
        responder_id: String,

        /// Path to Zenoh configuration file
        #[arg(long)]
        zenoh_config: Option<PathBuf>,

        /// Path to Python virtual environment
        #[arg(long)]
        python_env: Option<PathBuf>,

        /// Enable debug logging
        #[arg(long)]
        debug: bool,

        /// Enable Zenoh debug logging
        #[arg(long)]
        debug_zenoh: bool,
    },

    /// Create a Liaison FMU from an existing FMU
    MakeFmu {
        /// Path to the source FMU file
        fmu_path: PathBuf,

        /// Responder ID for Zenoh network
        responder_id: String,

        /// Path to Zenoh configuration file
        #[arg(long)]
        zenoh_config: Option<PathBuf>,

        /// Enable debug logging
        #[arg(long)]
        debug: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Serve {
            fmu_path,
            responder_id,
            zenoh_config,
            python_env,
            debug,
            debug_zenoh,
        } => {
            // Initialize logging
            init_logging(debug);

            if debug_zenoh {
                // Enable Zenoh debug logging
                std::env::set_var("RUST_LOG", "zenoh=debug");
            }

            info!("Starting Liaison server...");
            info!("FMU: {:?}", fmu_path);
            info!("Responder ID: {}", responder_id);

            if let Some(ref config) = zenoh_config {
                info!("Zenoh config: {:?}", config);
            }

            if let Some(ref py_env) = python_env {
                info!("Python environment: {:?}", py_env);
            }

            server::start_server(
                fmu_path,
                responder_id,
                zenoh_config,
                python_env,
                debug,
            )?;
        }

        Commands::MakeFmu {
            fmu_path,
            responder_id,
            zenoh_config,
            debug,
        } => {
            // Initialize logging
            init_logging(debug);

            info!("Creating Liaison FMU...");
            info!("Source FMU: {:?}", fmu_path);
            info!("Responder ID: {}", responder_id);

            if let Some(ref config) = zenoh_config {
                info!("Zenoh config: {:?}", config);
            }

            fmu_creator::make_fmu(
                fmu_path,
                responder_id,
                zenoh_config,
            )?;
        }
    }

    Ok(())
}

fn init_logging(debug: bool) {
    use tracing_subscriber::{EnvFilter, fmt};

    let filter = if debug {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}
