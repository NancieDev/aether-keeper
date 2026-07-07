# Aether Keeper API Reference

## Smart Contract Methods

### Task Management

#### `register_task(creator, target_contract, function_name, interval, bounty_per_exec, initial_funding) → u64`

Register a new automation task.

**Parameters**:
- `creator`: Address of the task creator (will be charged)
- `target_contract`: Contract to invoke (Address)
- `function_name`: Function to call (Symbol)
- `interval`: Execution interval in seconds (u64, must be > 0)
- `bounty_per_exec`: Payment per execution in stroops (i128, must be > 0)
- `initial_funding`: Initial bounty pool (i128, must be >= bounty_per_exec)

**Returns**: Task ID (u64)

**Errors**:
- `InvalidInterval`: interval must be > 0
- `InvalidBounty`: bounty_per_exec must be > 0
- `InsufficientFunding`: initial_funding < bounty_per_exec
- `TransferFailed`: Could not transfer funds from creator

**Example**:
```rust
let task_id = contract.register_task(
    creator_addr,
    target_contract_addr,
    Symbol::short("compound"),
    86400,  // 1 day
    500_000_000,  // 5 XLM
    5_000_000_000  // 50 XLM initial
)?;
```

---

#### `execute_task(keeper, task_id) → bool`

Execute a registered task.

**Parameters**:
- `keeper`: Address of the keeper (earns bounty)
- `task_id`: Task to execute (u64)

**Returns**: true if execution succeeded

**Errors**:
- `TaskNotFound`: Task doesn't exist
- `TaskNotReady`: Interval not elapsed
- `InsufficientFunds`: No funds remaining
- `ExecutionFailed`: Target contract invocation failed
- `TransferFailed`: Could not pay keeper bounty

**Example**:
```rust
let success = contract.execute_task(keeper_addr, 1)?;
```

---

#### `pause_task(creator, task_id) → ()`

Pause a task (stops execution, funds remain locked).

**Parameters**:
- `creator`: Task creator (must match original)
- `task_id`: Task to pause (u64)

**Errors**:
- `TaskNotFound`: Task doesn't exist
- `Unauthorized`: Not the task creator

---

#### `resume_task(creator, task_id) → ()`

Resume a paused task.

**Parameters**:
- `creator`: Task creator (must match original)
- `task_id`: Task to resume (u64)

**Errors**:
- `TaskNotFound`: Task doesn't exist
- `Unauthorized`: Not the task creator

---

#### `deposit(creator, task_id, amount) → ()`

Add funds to a task's bounty pool.

**Parameters**:
- `creator`: Task creator
- `task_id`: Task to fund
- `amount`: Amount to deposit in stroops (i128, must be > 0)

**Errors**:
- `TaskNotFound`: Task doesn't exist
- `InvalidAmount`: amount must be > 0
- `TransferFailed`: Could not transfer funds

---

#### `withdraw(creator, task_id, amount) → ()`

Withdraw funds from a task's bounty pool.

**Parameters**:
- `creator`: Task creator (must match original)
- `task_id`: Task to withdraw from
- `amount`: Amount to withdraw in stroops (i128)

**Returns**: Withdrawn amount

**Errors**:
- `TaskNotFound`: Task doesn't exist
- `Unauthorized`: Not the task creator
- `InsufficientFunds`: Task doesn't have that much remaining
- `TransferFailed`: Could not transfer funds back

---

#### `cancel_task(creator, task_id) → ()`

Cancel a task and return all remaining funds.

**Parameters**:
- `creator`: Task creator (must match original)
- `task_id`: Task to cancel (u64)

**Errors**:
- `TaskNotFound`: Task doesn't exist
- `Unauthorized`: Not the task creator
- `TransferFailed`: Could not return funds

---

### Query Methods

#### `get_task(task_id) → TaskConfig`

Retrieve task configuration and state.

**Parameters**:
- `task_id`: Task ID (u64)

**Returns**: TaskConfig struct
```rust
pub struct TaskConfig {
    pub id: u64,
    pub creator: Address,
    pub target_contract: Address,
    pub function_name: Symbol,
    pub interval: u64,
    pub bounty_per_exec: i128,
    pub remaining_funds: i128,
    pub is_active: bool,
    pub last_executed: u64,
    pub execution_count: u64,
}
```

**Errors**:
- `TaskNotFound`: Task doesn't exist

---

#### `get_execution_record(task_id, index) → ExecutionRecord`

Retrieve historical execution data.

**Parameters**:
- `task_id`: Task ID (u64)
- `index`: Execution index (u64)

**Returns**: ExecutionRecord struct
```rust
pub struct ExecutionRecord {
    pub task_id: u64,
    pub keeper: Address,
    pub executed_at: u64,
    pub next_eligible: u64,
    pub bounty_paid: i128,
    pub success: bool,
}
```

**Errors**:
- `TaskNotFound`: Task doesn't exist
- `RecordNotFound`: Execution record not found

---

## Keeper CLI Commands

### Configuration

```bash
# Initialize keeper with secret key and RPC endpoint
keeper init --secret-key SABC... --rpc-url https://soroban-testnet.stellar.org --contract-id CABC...

# Show current configuration
keeper config show

# Update RPC endpoint
keeper config set-rpc https://new-rpc-url
```

### Operations

```bash
# Start polling and executing tasks
keeper start

# Check keeper account balance
keeper balance

# List tasks (active, paused, completed)
keeper list --status active
keeper list --status paused
keeper list --status all

# View execution metrics
keeper metrics
keeper metrics --format json

# Claim a specific task (manual execution)
keeper execute <task_id>

# Check pending transactions
keeper pending
```

### Monitoring

```bash
# Stream live execution logs
keeper logs

# Get execution history
keeper history <task_id>
keeper history --limit 50

# Health check
keeper health
```

---

## Frontend Hooks

### useContract()

Custom React hook for interacting with Aether Keeper contract.

```typescript
const {
  // State
  isConnected,
  pubKey,
  isLoading,
  error,
  
  // Methods
  connect,
  disconnect,
  registerTask,
  pauseTask,
  resumeTask,
  deposit,
  withdraw,
  cancelTask,
  getTasks,
  
} = useContract();
```

**Usage**:
```typescript
// Connect wallet
await connect();

// Register a task
const taskId = await registerTask({
  targetContract: 'CABC...',
  functionName: 'compound',
  interval: 86400,
  bountyPerExec: 500_000_000,
  initialFunding: 5_000_000_000,
});

// List creator's tasks
const tasks = await getTasks();
```

---

## Event Types

### TaskRegistered
```rust
pub struct TaskRegistered {
    pub id: u64,
    pub creator: Address,
    pub target_contract: Address,
    pub function_name: Symbol,
    pub interval: u64,
    pub bounty_per_exec: i128,
}
```

### TaskExecuted
```rust
pub struct TaskExecuted {
    pub id: u64,
    pub keeper: Address,
    pub executed_at: u64,
    pub next_eligible: u64,
    pub bounty_paid: i128,
}
```

### TaskPaused
```rust
pub struct TaskPaused {
    pub id: u64,
    pub creator: Address,
}
```

### TaskResumed
```rust
pub struct TaskResumed {
    pub id: u64,
    pub creator: Address,
}
```

### FundsDeposited
```rust
pub struct FundsDeposited {
    pub id: u64,
    pub creator: Address,
    pub amount: i128,
    pub new_balance: i128,
}
```

### TaskCanceled
```rust
pub struct TaskCanceled {
    pub id: u64,
    pub creator: Address,
    pub refunded: i128,
}
```

---

## Error Codes

| Code | Meaning | Resolution |
|------|---------|-----------|
| `InvalidInterval` | Interval must be > 0 | Use interval >= 1 second |
| `InvalidBounty` | Bounty must be > 0 | Specify bounty > 0 |
| `InsufficientFunding` | Initial funding < bounty | Increase initial funding |
| `TaskNotFound` | Task ID doesn't exist | Verify task ID |
| `TaskNotReady` | Interval not elapsed | Wait for next interval |
| `InsufficientFunds` | Task balance exhausted | Deposit more funds |
| `Unauthorized` | Not task creator | Use correct creator address |
| `ExecutionFailed` | Target contract failed | Check target contract |
| `TransferFailed` | Token transfer failed | Check wallet balance/allowance |

---

## Examples

### Register Task (CLI)
```bash
keeper exec register_task \
  --target-contract CABC123... \
  --function-name compound_yield \
  --interval 86400 \
  --bounty 500000000 \
  --funding 5000000000
```

### Register Task (Frontend)
```typescript
const taskId = await registerTask({
  targetContract: 'CABC...',
  functionName: 'compound_yield',
  interval: 86400,
  bountyPerExec: 500_000_000,
  initialFunding: 5_000_000_000,
});
console.log(`Task registered: ${taskId}`);
```

### Execute Task (Keeper)
```bash
# Keeper daemon automatically executes ready tasks
keeper start

# Or manually trigger
keeper execute 1
```

---

## Rate Limits & Constraints

- Maximum tasks per creator: unlimited
- Maximum concurrent executions per keeper: configurable (default: 10)
- Minimum interval: 60 seconds (1 minute)
- Maximum interval: 31,536,000 seconds (1 year)
- Bounty precision: stroops (10^-7 XLM)

---

## Support

- **Documentation**: See [README.md](./README.md) and [ARCHITECTURE.md](./ARCHITECTURE.md)
- **Issues**: [GitHub Issues](https://github.com/NancieDev/aether-keeper/issues)
- **Quick Start**: [QUICKSTART.md](./QUICKSTART.md)
