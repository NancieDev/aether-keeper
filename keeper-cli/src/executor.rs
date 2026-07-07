use crate::config::KeeperConfig;
use anyhow::{anyhow, Context, Result};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info, warn, error};
use std::collections::HashMap;

/// Task executor responsible for polling and executing tasks
pub struct TaskExecutor {
    config: KeeperConfig,
}

/// Minimal TaskConfig representation for executor logic
#[derive(Clone, Debug)]
pub struct TaskInfo {
    pub id: u64,
    pub is_active: bool,
    pub interval: u64,
    pub last_executed_at: u64,
    pub bounty_per_exec: i128,
    pub remaining_funds: i128,
    pub execution_count: u64,
    pub max_executions: u64,
}

impl TaskExecutor {
    pub fn new(config: &KeeperConfig) -> Result<Self> {
        // Validate config
        if config.secret_key.is_empty() {
            return Err(anyhow!("secret_key is required"));
        }
        if config.rpc_url.is_empty() {
            return Err(anyhow!("rpc_url is required"));
        }
        if config.contract_id.is_empty() {
            return Err(anyhow!("contract_id is required"));
        }

        Ok(Self {
            config: config.clone(),
        })
    }

    /// Scan contract for executable tasks and execute them
    pub async fn scan_and_execute_tasks(&self) -> Result<u32> {
        debug!("Scanning for executable tasks...");
        
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .context("Failed to get current time")?
            .as_secs();

        // Step 1: Query all tasks from contract
        let all_tasks = self.query_all_tasks().await?;
        debug!("Found {} total tasks", all_tasks.len());

        // Step 2: Filter executable tasks
        let executable_tasks: Vec<_> = all_tasks
            .iter()
            .filter(|task| {
                task.is_active
                    && current_time >= task.last_executed_at + task.interval
                    && task.remaining_funds >= task.bounty_per_exec
                    && (task.max_executions == 0 || task.execution_count < task.max_executions)
            })
            .collect();

        debug!("Found {} executable tasks", executable_tasks.len());

        if executable_tasks.is_empty() {
            return Ok(0);
        }

        // Step 3: Execute up to max_concurrent_tasks in parallel
        let mut executed = 0u32;
        let max_concurrent = self.config.max_concurrent_tasks.min(executable_tasks.len() as u32);

        for chunk in executable_tasks.chunks(max_concurrent as usize) {
            let futures: Vec<_> = chunk
                .iter()
                .map(|task| self.execute_task_internal(task.id))
                .collect();

            for future in futures {
                match future.await {
                    Ok(_) => executed += 1,
                    Err(e) => warn!("Task execution failed: {}", e),
                }
            }
        }

        info!("Executed {} tasks successfully", executed);
        Ok(executed)
    }

    /// Execute a specific task with retry logic
    pub async fn execute_task(&self, task_id: u64) -> Result<()> {
        info!("Executing task {}", task_id);
        self.execute_task_internal(task_id).await
    }

    /// Internal task execution with exponential backoff retries
    async fn execute_task_internal(&self, task_id: u64) -> Result<()> {
        let max_retries = self.config.retry_max_attempts;
        let mut retry_delay = Duration::from_millis(1000); // Start with 1s

        for attempt in 0..max_retries {
            match self.submit_execute_task_tx(task_id).await {
                Ok(tx_hash) => {
                    // Wait for confirmation
                    match self.wait_for_confirmation(&tx_hash, 30).await {
                        Ok(confirmed) => {
                            if confirmed {
                                info!("Task {} executed successfully (tx: {})", task_id, &tx_hash[..16]);
                                return Ok(());
                            } else {
                                warn!("Task {} execution timed out after confirmation polling", task_id);
                                return Err(anyhow!("Confirmation timeout"));
                            }
                        }
                        Err(e) => {
                            warn!("Confirmation polling failed for task {}: {}", task_id, e);
                            return Err(e);
                        }
                    }
                }
                Err(e) => {
                    // Determine if error is transient or permanent
                    let is_transient = e.to_string().contains("timeout")
                        || e.to_string().contains("connection")
                        || e.to_string().contains("503")
                        || e.to_string().contains("502");

                    if is_transient && attempt < max_retries - 1 {
                        warn!(
                            "Task {} submission failed (transient, attempt {}/{}): {}. Retrying in {:?}",
                            task_id, attempt + 1, max_retries, e, retry_delay
                        );
                        sleep(retry_delay).await;
                        retry_delay = retry_delay.saturating_mul(2); // Exponential backoff
                    } else {
                        error!("Task {} execution failed permanently: {}", task_id, e);
                        return Err(e);
                    }
                }
            }
        }

        Err(anyhow!(
            "Task {} failed after {} retry attempts",
            task_id,
            max_retries
        ))
    }

    /// Query all tasks from contract
    async fn query_all_tasks(&self) -> Result<Vec<TaskInfo>> {
        // In a real implementation, this would:
        // 1. Query contract.get_task() for each task ID
        // 2. Or use a contract function that returns all tasks
        // 3. Parse Soroban RPC responses
        //
        // For now, return placeholder. Production would integrate:
        // - stellar_sdk for RPC calls
        // - soroban SDK types for contract interaction
        
        debug!("Querying tasks from contract {}", &self.config.contract_id);
        
        // TODO: Implement full Soroban RPC integration
        // This requires:
        // - stellar-rs crate (Soroban SDK)
        // - Contract bindings generated from ABI
        // - RPC client initialization with config.rpc_url
        
        Ok(vec![])
    }

    /// Submit execute_task transaction to contract
    async fn submit_execute_task_tx(&self, task_id: u64) -> Result<String> {
        info!("Submitting execute_task transaction for task {}", task_id);

        // TODO: Implement full Soroban transaction building
        // This requires:
        // 1. Initialize Soroban RPC client
        // 2. Get keeper public key from secret_key
        // 3. Build invoke_contract transaction:
        //    - Contract: self.config.contract_id
        //    - Function: "execute_task"
        //    - Args: [keeper_address, task_id]
        // 4. Sign transaction with keeper's secret key
        // 5. Submit to Soroban RPC
        // 6. Return transaction hash
        
        // Placeholder: return mock tx hash
        Ok(format!("tx_{}", task_id))
    }

    /// Poll for transaction confirmation
    async fn wait_for_confirmation(&self, tx_hash: &str, max_secs: u64) -> Result<bool> {
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(max_secs);

        loop {
            if start.elapsed() > timeout {
                return Ok(false); // Timeout
            }

            // TODO: Query Soroban RPC for transaction status
            // soroban.getTransaction(tx_hash) -> returns status
            // If status == "SUCCESS" -> return Ok(true)
            // If status == "FAILED" -> return Err(...)
            
            sleep(Duration::from_millis(500)).await;
        }
    }

    /// List tasks with optional status filter
    pub async fn list_tasks(&self, status: Option<&str>) -> Result<()> {
        info!("Listing tasks (status: {:?})", status);

        let tasks = self.query_all_tasks().await?;

        if tasks.is_empty() {
            println!("No tasks found");
            return Ok(());
        }

        // Print table header
        println!(
            "{:<5} {:<10} {:<15} {:<15} {:<20}",
            "ID", "Status", "Last Exec", "Remaining", "Bounty"
        );
        println!("{:-<65}", "");

        // Filter by status if provided
        let filtered: Vec<_> = tasks
            .iter()
            .filter(|t| {
                status.is_none()
                    || (status == Some("active") && t.is_active)
                    || (status == Some("paused") && !t.is_active)
            })
            .collect();

        // Print each task
        for task in filtered {
            let status_str = if task.is_active { "active" } else { "paused" };
            println!(
                "{:<5} {:<10} {:<15} {:<15} {:<20}",
                task.id, status_str, task.last_executed_at, task.remaining_funds, task.bounty_per_exec
            );
        }

        Ok(())
    }

    /// Get detailed task information
    pub async fn get_task_info(&self, task_id: u64) -> Result<()> {
        info!("Fetching info for task {}", task_id);

        let tasks = self.query_all_tasks().await?;
        let task = tasks
            .iter()
            .find(|t| t.id == task_id)
            .ok_or_else(|| anyhow!("Task {} not found", task_id))?;

        println!("Task Information");
        println!("{:-<40}", "");
        println!("ID:                  {}", task.id);
        println!("Status:              {}", if task.is_active { "active" } else { "paused" });
        println!("Interval (seconds):  {}", task.interval);
        println!("Last Executed:       {}", task.last_executed_at);
        println!("Bounty per Exec:     {}", task.bounty_per_exec);
        println!("Remaining Funds:     {}", task.remaining_funds);
        println!("Execution Count:     {}", task.execution_count);
        println!("Max Executions:      {}", if task.max_executions == 0 { "unlimited".to_string() } else { task.max_executions.to_string() });

        Ok(())
    }

    /// Check keeper's balance
    pub async fn check_balance(&self) -> Result<()> {
        info!("Checking keeper balance...");

        // TODO: Query keeper's token balance via Soroban RPC
        // 1. Get keeper public key from secret_key
        // 2. Query token contract for balance(keeper_addr)
        // 3. Display with proper denomination
        
        println!("Keeper Balance: <not yet implemented>");

        Ok(())
    }
}
