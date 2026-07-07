#![no_std]

use soroban_sdk::{contracttype, Address, Symbol, Val, Vec};

/// Configuration for an automated task
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskConfig {
    /// Address that created and funds the task
    pub creator: Address,
    /// Target contract to invoke
    pub target_contract: Address,
    /// Function name to call on target contract
    pub function: Symbol,
    /// Arguments to pass to the function
    pub args: Vec<Val>,
    /// Minimum seconds between executions (interval constraint)
    pub interval: u64,
    /// Last execution timestamp (prevents execution before interval elapses)
    pub last_executed_at: u64,
    /// Bounty paid to keeper per execution (in token units)
    pub bounty_per_exec: i128,
    /// Remaining funds available for bounty payments
    pub remaining_funds: i128,
    /// Token address used for bounty payments (typically XLM)
    pub token: Address,
    /// Whether the task is active (allows pause without deletion)
    pub is_active: bool,
    /// Maximum execution count (0 = unlimited)
    pub max_executions: u64,
    /// Current execution count
    pub execution_count: u64,
}

/// Storage key enumeration for contract state
#[contracttype]
pub enum DataKey {
    /// Admin address (singleton)
    Admin,
    /// Global task counter for generating unique IDs
    TaskCounter,
    /// Individual task configuration by ID
    Task(u64),
    /// Task execution history (ledger entries for audit)
    TaskExecutionHistory(u64, u64), // (task_id, execution_index)
}

/// Task execution record for audit trail
#[contracttype]
#[derive(Clone, Debug)]
pub struct ExecutionRecord {
    pub task_id: u64,
    pub executor: Address,
    pub executed_at: u64,
    pub bounty_paid: i128,
}
