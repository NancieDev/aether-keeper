#![no_std]

use soroban_sdk::{symbol_short, Address, Env, Symbol};

/// Emitted when a task is registered
pub fn emit_task_registered(env: &Env, task_id: u64, creator: Address, bounty_per_exec: i128) {
    env.events().publish(
        (Symbol::new(env, "aether_task_registered"),),
        (task_id, creator, bounty_per_exec),
    );
}

/// Emitted when a task is executed
pub fn emit_task_executed(env: &Env, task_id: u64, keeper: Address, bounty_paid: i128) {
    env.events().publish(
        (Symbol::new(env, "aether_task_executed"),),
        (task_id, keeper, bounty_paid),
    );
}

/// Emitted when a task is paused
pub fn emit_task_paused(env: &Env, task_id: u64, reason: Symbol) {
    env.events().publish(
        (Symbol::new(env, "aether_task_paused"),),
        (task_id, reason),
    );
}

/// Emitted when a task is resumed
pub fn emit_task_resumed(env: &Env, task_id: u64) {
    env.events().publish(
        (Symbol::new(env, "aether_task_resumed"),),
        task_id,
    );
}

/// Emitted when a task is cancelled
pub fn emit_task_cancelled(env: &Env, task_id: u64, reason: Symbol) {
    env.events().publish(
        (Symbol::new(env, "aether_task_cancelled"),),
        (task_id, reason),
    );
}

/// Emitted when funds are withdrawn from a task
pub fn emit_funds_withdrawn(env: &Env, task_id: u64, amount: i128, recipient: Address) {
    env.events().publish(
        (Symbol::new(env, "aether_funds_withdrawn"),),
        (task_id, amount, recipient),
    );
}

/// Emitted when funds are deposited to a task
pub fn emit_funds_deposited(env: &Env, task_id: u64, amount: i128, depositor: Address) {
    env.events().publish(
        (Symbol::new(env, "aether_funds_deposited"),),
        (task_id, amount, depositor),
    );
}

/// Emitted on execution error
pub fn emit_execution_error(env: &Env, task_id: u64, error_code: u32) {
    env.events().publish(
        (Symbol::new(env, "aether_execution_error"),),
        (task_id, error_code),
    );
}
