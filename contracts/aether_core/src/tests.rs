#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env, IntoVal, Symbol,
};

fn create_test_env() -> (Env, Address, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    // Create mock token
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract(token_admin.clone());

    // Create contract
    let contract_id = env.register_contract(None, AetherKeeper);
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let keeper = Address::generate(&env);

    // Mint tokens to creator
    let token_admin_client = token::StellarAssetClient::new(&env, &token_contract);
    token_admin_client.mint(&creator, &10000);

    (env, contract_id, admin, creator, keeper, token_contract)
}

#[test]
fn test_initialize() {
    let (env, contract_id, admin, _, _) = create_test_env();
    let client = AetherKeeperClient::new(&env, &contract_id);

    let result = client.initialize(&admin);
    assert!(result.is_ok());

    // Test already initialized error
    let result = client.initialize(&admin);
    assert_eq!(result, Err(Ok(ContractError::AlreadyInitialized)));
}

#[test]
fn test_register_task() {
    let (env, contract_id, admin, creator, _, token_contract) = create_test_env();
    let client = AetherKeeperClient::new(&env, &contract_id);
    let token_client = token::Client::new(&env, &token_contract);

    client.initialize(&admin).unwrap();

    let target_contract = Address::generate(&env);
    let function = Symbol::new(&env, "ping");
    let args = Vec::new(&env);

    let task_id = client
        .register_task(
            &creator,
            &target_contract,
            &function,
            &args,
            60,      // interval
            10,      // bounty_per_exec
            100,     // initial_funding
            &token_contract,
        )
        .unwrap();

    assert_eq!(task_id, 0);

    // Verify funds transferred
    assert_eq!(token_client.balance(&creator), 9900);
    assert_eq!(token_client.balance(&contract_id), 100);

    // Verify task can be retrieved
    let task = client.get_task(&task_id).unwrap();
    assert_eq!(task.creator, creator);
    assert_eq!(task.bounty_per_exec, 10);
    assert_eq!(task.remaining_funds, 100);
    assert!(task.is_active);
    assert_eq!(task.execution_count, 0);
}

#[test]
fn test_register_task_insufficient_funding() {
    let (env, contract_id, admin, creator, _, token_contract) = create_test_env();
    let client = AetherKeeperClient::new(&env, &contract_id);

    client.initialize(&admin).unwrap();

    let target_contract = Address::generate(&env);
    let function = Symbol::new(&env, "ping");
    let args = Vec::new(&env);

    let result = client.register_task(
        &creator,
        &target_contract,
        &function,
        &args,
        60,  // interval
        10,  // bounty_per_exec
        5,   // initial_funding (less than bounty)
        &token_contract,
    );

    assert_eq!(result, Err(Ok(ContractError::InsufficientFunding)));
}

#[test]
fn test_execute_task_success() {
    let (env, contract_id, admin, creator, keeper, token_contract) = create_test_env();
    let client = AetherKeeperClient::new(&env, &contract_id);
    let token_client = token::Client::new(&env, &token_contract);

    client.initialize(&admin).unwrap();

    let target_contract = Address::generate(&env);
    let function = Symbol::new(&env, "ping");
    let args = Vec::new(&env);

    let task_id = client
        .register_task(
            &creator,
            &target_contract,
            &function,
            &args,
            60,
            10,
            100,
            &token_contract,
        )
        .unwrap();

    // Advance time
    env.ledger().set(soroban_sdk::testutils::LedgerInfo {
        timestamp: 100,
        protocol_version: 20,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: 1000000,
    });

    // Execute task
    let result = client.execute_task(&keeper, &task_id);
    assert!(result.is_ok());

    // Verify bounty paid
    assert_eq!(token_client.balance(&keeper), 10);
    assert_eq!(token_client.balance(&contract_id), 90);

    // Verify task state updated
    let task = client.get_task(&task_id).unwrap();
    assert_eq!(task.execution_count, 1);
    assert_eq!(task.remaining_funds, 90);
    assert_eq!(task.last_executed_at, 100);
}

#[test]
fn test_execute_task_interval_not_met() {
    let (env, contract_id, admin, creator, keeper, token_contract) = create_test_env();
    let client = AetherKeeperClient::new(&env, &contract_id);

    client.initialize(&admin).unwrap();

    let target_contract = Address::generate(&env);
    let function = Symbol::new(&env, "ping");
    let args = Vec::new(&env);

    let task_id = client
        .register_task(
            &creator,
            &target_contract,
            &function,
            &args,
            60,
            10,
            100,
            &token_contract,
        )
        .unwrap();

    // Try to execute immediately (interval not met)
    let result = client.execute_task(&keeper, &task_id);
    assert_eq!(result, Err(Ok(ContractError::IntervalNotMet)));
}

#[test]
fn test_pause_and_resume() {
    let (env, contract_id, admin, creator, _, token_contract) = create_test_env();
    let client = AetherKeeperClient::new(&env, &contract_id);

    client.initialize(&admin).unwrap();

    let target_contract = Address::generate(&env);
    let function = Symbol::new(&env, "ping");
    let args = Vec::new(&env);

    let task_id = client
        .register_task(
            &creator,
            &target_contract,
            &function,
            &args,
            60,
            10,
            100,
            &token_contract,
        )
        .unwrap();

    // Pause task
    let result = client.pause_task(&task_id);
    assert!(result.is_ok());

    let task = client.get_task(&task_id).unwrap();
    assert!(!task.is_active);

    // Resume task
    let result = client.resume_task(&task_id);
    assert!(result.is_ok());

    let task = client.get_task(&task_id).unwrap();
    assert!(task.is_active);
}

#[test]
fn test_deposit_funds() {
    let (env, contract_id, admin, creator, _, token_contract) = create_test_env();
    let client = AetherKeeperClient::new(&env, &contract_id);
    let token_client = token::Client::new(&env, &token_contract);

    client.initialize(&admin).unwrap();

    let target_contract = Address::generate(&env);
    let function = Symbol::new(&env, "ping");
    let args = Vec::new(&env);

    let task_id = client
        .register_task(
            &creator,
            &target_contract,
            &function,
            &args,
            60,
            10,
            100,
            &token_contract,
        )
        .unwrap();

    // Deposit additional funds
    let result = client.deposit_funds(&task_id, &50);
    assert!(result.is_ok());

    let task = client.get_task(&task_id).unwrap();
    assert_eq!(task.remaining_funds, 150);
    assert_eq!(token_client.balance(&contract_id), 150);
}

#[test]
fn test_withdraw_funds() {
    let (env, contract_id, admin, creator, _, token_contract) = create_test_env();
    let client = AetherKeeperClient::new(&env, &contract_id);
    let token_client = token::Client::new(&env, &token_contract);

    client.initialize(&admin).unwrap();

    let target_contract = Address::generate(&env);
    let function = Symbol::new(&env, "ping");
    let args = Vec::new(&env);

    let task_id = client
        .register_task(
            &creator,
            &target_contract,
            &function,
            &args,
            60,
            10,
            100,
            &token_contract,
        )
        .unwrap();

    // Withdraw funds
    let result = client.withdraw_funds(&task_id, &30);
    assert!(result.is_ok());

    let task = client.get_task(&task_id).unwrap();
    assert_eq!(task.remaining_funds, 70);
    assert!(!task.is_active); // Should be paused when withdrawing
    assert_eq!(token_client.balance(&creator), 9930);
}

#[test]
fn test_cancel_task() {
    let (env, contract_id, admin, creator, _, token_contract) = create_test_env();
    let client = AetherKeeperClient::new(&env, &contract_id);
    let token_client = token::Client::new(&env, &token_contract);

    client.initialize(&admin).unwrap();

    let target_contract = Address::generate(&env);
    let function = Symbol::new(&env, "ping");
    let args = Vec::new(&env);

    let task_id = client
        .register_task(
            &creator,
            &target_contract,
            &function,
            &args,
            60,
            10,
            100,
            &token_contract,
        )
        .unwrap();

    // Cancel task
    let result = client.cancel_task(&task_id);
    assert!(result.is_ok());

    // Funds should be returned
    assert_eq!(token_client.balance(&creator), 10000);
    assert_eq!(token_client.balance(&contract_id), 0);

    // Task should not exist
    let result = client.get_task(&task_id);
    assert!(result.is_err());
}
