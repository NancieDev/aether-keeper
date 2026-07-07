use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

mod config;
mod executor;
mod keeper;

use config::KeeperConfig;
use executor::TaskExecutor;

#[derive(Parser)]
#[command(name = "keeper")]
#[command(about = "Aether Keeper CLI - Execute decentralized automation tasks", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Configuration file path
    #[arg(global = true, long, short)]
    config: Option<PathBuf>,

    /// Verbose logging
    #[arg(global = true, long, short)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize keeper configuration
    Init {
        /// Keeper name/identifier
        #[arg(short, long)]
        name: String,

        /// Private key (secret seed)
        #[arg(short, long)]
        secret_key: String,

        /// Soroban RPC URL
        #[arg(short, long)]
        rpc_url: String,

        /// Aether contract ID
        #[arg(short, long)]
        contract_id: String,

        /// Network (testnet/mainnet)
        #[arg(short, long, default_value = "testnet")]
        network: String,
    },

    /// Start keeper daemon
    Start,

    /// Execute a single task
    Execute {
        /// Task ID
        #[arg(short, long)]
        task_id: u64,

        /// One-time execution (don't loop)
        #[arg(short, long)]
        once: bool,
    },

    /// List available tasks
    List {
        /// Filter by status (active/paused/completed)
        #[arg(short, long)]
        status: Option<String>,
    },

    /// Get task details
    Info {
        /// Task ID
        #[arg(short, long)]
        task_id: u64,
    },

    /// Query keeper balance
    Balance,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    let config_path = cli.config.unwrap_or_else(|| {
        let home = dirs::home_dir().unwrap_or_default();
        home.join(".aether")
            .join("keeper-config.json")
    });

    match cli.command {
        Commands::Init {
            name,
            secret_key,
            rpc_url,
            contract_id,
            network,
        } => {
            init_keeper(&config_path, name, secret_key, rpc_url, contract_id, network)?;
        }

        Commands::Start => {
            let config = load_config(&config_path)?;
            run_keeper_daemon(&config).await?;
        }

        Commands::Execute { task_id, once } => {
            let config = load_config(&config_path)?;
            let executor = TaskExecutor::new(&config)?;
            executor.execute_task(task_id).await?;

            if !once {
                // Watch mode: re-execute when interval allows
                loop {
                    sleep(Duration::from_secs(30)).await;
                    if let Err(e) = executor.execute_task(task_id).await {
                        warn!("Execution failed: {}", e);
                    }
                }
            }
        }

        Commands::List { status } => {
            let config = load_config(&config_path)?;
            let executor = TaskExecutor::new(&config)?;
            executor.list_tasks(status.as_deref()).await?;
        }

        Commands::Info { task_id } => {
            let config = load_config(&config_path)?;
            let executor = TaskExecutor::new(&config)?;
            executor.get_task_info(task_id).await?;
        }

        Commands::Balance => {
            let config = load_config(&config_path)?;
            let executor = TaskExecutor::new(&config)?;
            executor.check_balance().await?;
        }
    }

    Ok(())
}

fn init_keeper(
    config_path: &PathBuf,
    name: String,
    secret_key: String,
    rpc_url: String,
    contract_id: String,
    network: String,
) -> Result<()> {
    // Create config directory
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let config = KeeperConfig {
        name,
        secret_key,
        rpc_url,
        contract_id,
        network,
        poll_interval_secs: 60,
        max_concurrent_tasks: 5,
    };

    let json = serde_json::to_string_pretty(&config)?;
    fs::write(config_path, json)?;

    info!("Keeper initialized at {:?}", config_path);
    Ok(())
}

fn load_config(path: &PathBuf) -> Result<KeeperConfig> {
    let content = fs::read_to_string(path)
        .map_err(|e| anyhow!("Failed to read config: {}. Run 'keeper init' first.", e))?;
    
    serde_json::from_str(&content)
        .map_err(|e| anyhow!("Invalid config format: {}", e))
}

async fn run_keeper_daemon(config: &KeeperConfig) -> Result<()> {
    info!("Starting keeper daemon: {}", config.name);
    info!("Contract: {}", config.contract_id);
    info!("Poll interval: {}s", config.poll_interval_secs);

    let executor = TaskExecutor::new(config)?;

    loop {
        match executor.scan_and_execute_tasks().await {
            Ok(count) => {
                if count > 0 {
                    info!("Executed {} tasks", count);
                }
            }
            Err(e) => {
                error!("Error executing tasks: {}", e);
            }
        }

        sleep(Duration::from_secs(config.poll_interval_secs)).await;
    }
}
