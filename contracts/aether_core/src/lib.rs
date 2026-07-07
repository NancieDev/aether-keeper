#![no_std]

mod storage;
mod events;

use soroban_sdk::{contract, contractimpl, token, Address, Env, Symbol, Val, Vec};
use storage::{DataKey, TaskConfig, ExecutionRecord};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InsufficientFunding = 4,
    TaskNotFound = 5,
    IntervalNotMet = 6,
    InsufficientBountyFunds = 7,
    TaskInactive = 8,
    ExecutionLimitReached = 9,
    InvalidInterval = 10,
    InvalidBounty = 11,
    ContractInvocationFailed = 12,
    TransferFailed = 13,
}

impl From<ContractError> for u32 {
    fn from(err: ContractError) -> u32 {
        err as u32
    }
}

#[contract]
pub struct AetherKeeper;

#[contractimpl]
impl AetherKeeper {
    /// Initialize the Keeper Registry with an admin address
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `admin` - Address designated as contract administrator
    ///
    /// # Errors
    /// Returns `AlreadyInitialized` if already initialized
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }

        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TaskCounter, &0u64);

        Ok(())
    }

    /// Register a new automation task with initial funding
    ///
    /// # Arguments
    /// * `creator` - Address creating and funding the task
    /// * `target_contract` - Address of contract to invoke
    /// * `function` - Name of function to call
    /// * `args` - Arguments to pass to function
    /// * `interval` - Minimum seconds between executions
    /// * `bounty_per_exec` - Amount paid to keeper per execution
    /// * `initial_funding` - Total bounty pool
    /// * `token` - Token address for payments
    ///
    /// # Returns
    /// Task ID (u64)
    pub fn register_task(
        env: Env,
        creator: Address,
        target_contract: Address,
        function: Symbol,
        args: Vec<Val>,
        interval: u64,
        bounty_per_exec: i128,
        initial_funding: i128,
        token: Address,
    ) -> Result<u64, ContractError> {
        creator.require_auth();

        // Validation
        if interval == 0 {
            return Err(ContractError::InvalidInterval);
        }
        if interval > 315_360_000 { // Max ~10 years
            return Err(ContractError::InvalidInterval);
        }
        if bounty_per_exec <= 0 || bounty_per_exec > i128::MAX / 1_000_000 {
            return Err(ContractError::InvalidBounty);
        }
        if initial_funding < bounty_per_exec {
            return Err(ContractError::InsufficientFunding);
        }

        // Transfer funding from creator to contract
        let token_client = token::Client::new(&env, &token);
        token_client
            .transfer(&creator, &env.current_contract_address(), &initial_funding)
            .map_err(|_| ContractError::TransferFailed)?;

        // Generate Task ID
        let task_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TaskCounter)
            .unwrap_or(0);

        let config = TaskConfig {
            creator: creator.clone(),
            target_contract,
            function,
            args,
            interval,
            last_executed_at: 0,
            bounty_per_exec,
            remaining_funds: initial_funding,
            token,
            is_active: true,
            max_executions: 0, // unlimited
            execution_count: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Task(task_id), &config);
        env.storage()
            .instance()
            .set(&DataKey::TaskCounter, &(task_id + 1));

        events::emit_task_registered(&env, task_id, creator, bounty_per_exec);

        Ok(task_id)
    }

    /// Execute a registered task and pay the keeper
    ///
    /// # Arguments
    /// * `keeper` - Address of the keeper executing the task
    /// * `task_id` - ID of task to execute
    ///
    /// # Validation
    /// * Task must exist
    /// * Task must be active
    /// * Interval constraint must be satisfied
    /// * Sufficient funds must remain
    /// * Execution limit must not be reached
    pub fn execute_task(env: Env, keeper: Address, task_id: u64) -> Result<(), ContractError> {
        keeper.require_auth();

        let mut task: TaskConfig = env
            .storage()
            .persistent()
            .get(&DataKey::Task(task_id))
            .ok_or(ContractError::TaskNotFound)?;

        let current_time = env.ledger().timestamp();

        // Validation checks
        if !task.is_active {
            return Err(ContractError::TaskInactive);
        }

        if current_time < task.last_executed_at + task.interval {
            return Err(ContractError::IntervalNotMet);
        }

        if task.remaining_funds < task.bounty_per_exec {
            return Err(ContractError::InsufficientBountyFunds);
        }

        if task.max_executions > 0 && task.execution_count >= task.max_executions {
            return Err(ContractError::ExecutionLimitReached);
        }

        // Update state before external call (reentrancy protection)
        task.last_executed_at = current_time;
        task.remaining_funds -= task.bounty_per_exec;
        task.execution_count += 1;

        // Auto-pause if funds depleted
        if task.remaining_funds == 0 {
            task.is_active = false;
            events::emit_task_paused(
                &env,
                task_id,
                Symbol::new(&env, "funds_depleted"),
            );
        }

        env.storage()
            .persistent()
            .set(&DataKey::Task(task_id), &task);

        // Execute target contract with error handling
        let result = env.try_invoke_contract::<Val>(
            &task.target_contract,
            &task.function,
            task.args.clone(),
        );

        if result.is_err() {
            events::emit_execution_error(&env, task_id, ContractError::ContractInvocationFailed.into());
            return Err(ContractError::ContractInvocationFailed);
        }

        // Pay the keeper
        let token_client = token::Client::new(&env, &task.token);
        token_client.transfer(
            &env.current_contract_address(),
            &keeper,
            &task.bounty_per_exec,
        )
        .map_err(|_| ContractError::TransferFailed)?;

        events::emit_task_executed(&env, task_id, keeper, task.bounty_per_exec);

        Ok(())
    }

    /// Pause a task (creator only)
    pub fn pause_task(env: Env, task_id: u64) -> Result<(), ContractError> {
        let mut task: TaskConfig = env
            .storage()
            .persistent()
            .get(&DataKey::Task(task_id))
            .ok_or(ContractError::TaskNotFound)?;

        task.creator.require_auth();

        task.is_active = false;
        env.storage()
            .persistent()
            .set(&DataKey::Task(task_id), &task);

        events::emit_task_paused(&env, task_id, Symbol::new(&env, "creator_paused"));

        Ok(())
    }

    /// Resume a paused task (creator only)
    pub fn resume_task(env: Env, task_id: u64) -> Result<(), ContractError> {
        let mut task: TaskConfig = env
            .storage()
            .persistent()
            .get(&DataKey::Task(task_id))
            .ok_or(ContractError::TaskNotFound)?;

        task.creator.require_auth();

        task.is_active = true;
        env.storage()
            .persistent()
            .set(&DataKey::Task(task_id), &task);

        events::emit_task_resumed(&env, task_id);

        Ok(())
    }

    /// Deposit additional funds to a task (creator only)
    pub fn deposit_funds(env: Env, task_id: u64, amount: i128) -> Result<(), ContractError> {
        let mut task: TaskConfig = env
            .storage()
            .persistent()
            .get(&DataKey::Task(task_id))
            .ok_or(ContractError::TaskNotFound)?;

        task.creator.require_auth();

        if amount <= 0 {
            return Err(ContractError::InvalidBounty);
        }

        // Transfer funds from creator to contract
        let token_client = token::Client::new(&env, &task.token);
        token_client.transfer(&task.creator, &env.current_contract_address(), &amount)
            .map_err(|_| ContractError::TransferFailed)?;

        task.remaining_funds += amount;
        env.storage()
            .persistent()
            .set(&DataKey::Task(task_id), &task);

        events::emit_funds_deposited(&env, task_id, amount, task.creator);

        Ok(())
    }

    /// Withdraw unused funds from a task (creator only)
    pub fn withdraw_funds(env: Env, task_id: u64, amount: i128) -> Result<(), ContractError> {
        let mut task: TaskConfig = env
            .storage()
            .persistent()
            .get(&DataKey::Task(task_id))
            .ok_or(ContractError::TaskNotFound)?;

        task.creator.require_auth();

        if amount <= 0 || amount > task.remaining_funds {
            return Err(ContractError::InsufficientFunding);
        }

        // Pause task when withdrawing
        task.is_active = false;
        task.remaining_funds -= amount;

        env.storage()
            .persistent()
            .set(&DataKey::Task(task_id), &task);

        // Transfer funds back to creator
        let token_client = token::Client::new(&env, &task.token);
        token_client.transfer(&env.current_contract_address(), &task.creator, &amount)
            .map_err(|_| ContractError::TransferFailed)?;

        events::emit_funds_withdrawn(&env, task_id, amount, task.creator);

        Ok(())
    }

    /// Cancel a task and return remaining funds (creator only)
    pub fn cancel_task(env: Env, task_id: u64) -> Result<(), ContractError> {
        let task: TaskConfig = env
            .storage()
            .persistent()
            .get(&DataKey::Task(task_id))
            .ok_or(ContractError::TaskNotFound)?;

        task.creator.require_auth();

        // Return remaining funds to creator
        if task.remaining_funds > 0 {
            let token_client = token::Client::new(&env, &task.token);
            token_client.transfer(
                &env.current_contract_address(),
                &task.creator,
                &task.remaining_funds,
            )
            .map_err(|_| ContractError::TransferFailed)?;
        }

        env.storage().persistent().remove(&DataKey::Task(task_id));

        events::emit_task_cancelled(&env, task_id, Symbol::new(&env, "creator_cancelled"));

        Ok(())
    }

    /// Query task configuration
    pub fn get_task(env: Env, task_id: u64) -> Result<TaskConfig, ContractError> {
        env.storage()
            .persistent()
            .get(&DataKey::Task(task_id))
            .ok_or(ContractError::TaskNotFound)
    }
}

#[cfg(test)]
mod tests;
