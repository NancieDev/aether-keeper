# System Architecture

## Overview

Aether Keeper is a three-tier distributed system:

```
┌─────────────────────────────────────────────────────────────┐
│                     Stellar Blockchain                       │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  Soroban Smart Contract (aether_core)                │  │
│  │  - Task Registry & State                             │  │
│  │  - Access Control                                    │  │
│  │  - Token Transfer Coordination                       │  │
│  │  - Event Emission                                    │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              ▲
                    RPC ◄─────┼─────► RPC
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌──────────────┐      ┌──────────────┐      ┌──────────────┐
│ Keeper Node  │      │ Keeper Node  │      │ Keeper Node  │
│   (Daemon)   │      │   (Daemon)   │      │   (Daemon)   │
│              │      │              │      │              │
│ Poll Loop    │      │ Poll Loop    │      │ Poll Loop    │
│ Executor     │      │ Executor     │      │ Executor     │
│ Metrics DB   │      │ Metrics DB   │      │ Metrics DB   │
└──────────────┘      └──────────────┘      └──────────────┘
        │                     │                     │
        └─────────────────────┼─────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
 ┌─────────────┐      ┌──────────────┐      ┌──────────────┐
 │  Frontend   │      │   Indexer    │      │   Dashboard  │
 │  Next.js    │      │   Optional   │      │   (Web UI)   │
 │  React      │      │   Service    │      │   TypeScript │
 └─────────────┘      └──────────────┘      └──────────────┘
```

## Component Responsibilities

### 1. Smart Contract (`contracts/aether_core`)

**Purpose**: On-chain registry of automation tasks and payment coordination.

**Key Responsibilities**:
- Register new tasks with creator-specified parameters
- Store task configuration and state
- Verify execution eligibility (interval elapsed, funds available)
- Execute target contract invocations safely
- Coordinate token transfers (creator → contract, contract → keeper)
- Emit events for off-chain indexing

**Data Structures**:
- `TaskConfig`: Task metadata and state
- `DataKey`: Enumeration for persistent storage keys
- `ExecutionRecord`: Audit trail entry (optional historical storage)

**Security Model**:
- All state mutations require `require_auth()` from authorized principal
- Token transfers validated before state updates (transactional integrity)
- Reentrancy prevention via state-first updates
- Arithmetic overflow bounds checking

### 2. Keeper CLI (`keeper-cli`)

**Purpose**: Off-chain daemon that discovers and executes automation tasks.

**Key Responsibilities**:
- Long-running poll loop querying contract for executable tasks
- Soroban RPC integration for contract queries and transaction submission
- Transaction building and signing with keeper's secret key
- Retry logic with exponential backoff for transient failures
- Concurrent task execution (bounded by `max_concurrent_tasks` config)
- Metrics collection and local storage for operational visibility
- Command-line interface for one-off task execution and status queries

**Architecture**:
```
Keeper Daemon
├── Config Manager
│   ├── Load from ~/.aether/keeper-config.json
│   ├── Validate required fields
│   └── Parse environment overrides
├── Task Poller (runs every N seconds)
│   ├── Query contract for all tasks
│   ├── Filter executable tasks (interval met, funds available)
│   └── Queue ready tasks
├── Task Executor
│   ├── Get task details from contract
│   ├── Build execute_task transaction
│   ├── Sign with keeper's secret key
│   ├── Submit via Soroban RPC
│   ├── Poll for confirmation (max 30s)
│   └── Retry with backoff on failure
└── Metrics Collector
    ├── Store execution attempts in SQLite
    ├── Track success/failure rates
    └── Calculate earnings
```

**Dependencies**:
- `soroban-sdk`: Contract definitions and XDR types
- `soroban-rpc`: HTTP RPC client
- `tokio`: Async runtime
- `serde`/`serde_json`: Configuration serialization
- `tracing`: Structured logging
- `sqlx`/`rusqlite`: Local metrics database

### 3. Frontend (`frontend`)

**Purpose**: Web UI for task creators and managers to interact with the contract.

**Key Responsibilities**:
- Wallet connection (Freighter integration)
- Task creation form with input validation
- Real-time task list with status display
- Pause/resume/cancel task operations
- Fund management (deposit/withdraw)
- Error handling and user feedback
- Transaction confirmation flow

**Architecture**:
```
Next.js Application
├── Pages
│   └── / (index) - Main dashboard
├── Components
│   ├── Header - Wallet connection UI
│   ├── TaskDashboard - Task list & controls
│   ├── TaskTable - Tabular display
│   ├── CreateTaskModal - Task registration form
│   └── ErrorBoundary - Error handling wrapper
├── Hooks
│   ├── useWallet - Freighter integration
│   ├── useContract - Soroban RPC bindings
│   └── useTasks - Task list state
└── Store
    ├── wallet.ts - Wallet state (Zustand)
    └── tasks.ts - Task list state
```

**Key Features**:
- TypeScript for type safety
- Tailwind CSS for styling
- Zustand for lightweight state management
- Freighter wallet integration via `@stellar/freighter-api`
- Soroban RPC client via `@stellar/stellar-sdk`

## Data Flow

### Task Creation Flow

```
1. User fills CreateTaskModal form
   ├─ Target contract address
   ├─ Function name
   ├─ Interval (seconds)
   ├─ Bounty per execution
   └─ Initial funding amount

2. Frontend validates inputs
   ├─ Address format validation
   ├─ Numeric bounds checking
   └─ Sufficient wallet balance

3. Frontend builds Soroban transaction
   ├─ Create register_task invocation
   ├─ Serialize arguments (u64, i128, Address, Symbol, Vec<Val>)
   └─ Attach token transfer for initial funding

4. Freighter signs transaction
   ├─ User confirms in wallet UI
   └─ Returns signed XDR

5. Frontend submits via Soroban RPC
   ├─ POST to RPC endpoint
   └─ Get transaction hash

6. Frontend polls for confirmation
   ├─ Check transaction status every 1s
   ├─ Timeout after 30s
   └─ Display success/error

7. On success, refresh task list
   ├─ Frontend queries new task via get_task()
   └─ Display in dashboard
```

### Task Execution Flow

```
1. Keeper daemon polling loop (every 60s)
   └─ Query contract: "Get all tasks"

2. Contract returns task list via RPC
   ├─ For each task:
   │  ├─ Check is_active == true
   │  ├─ Check remaining_funds >= bounty_per_exec
   │  ├─ Check (now - last_executed_at) >= interval
   │  └─ If all true, add to executable queue

3. Keeper executor processes queue
   ├─ For each executable task (respecting max_concurrent_tasks):
   │  ├─ Build execute_task transaction
   │  ├─ Sign with keeper's secret key
   │  ├─ Submit via RPC
   │  ├─ Poll for confirmation (max 30s)
   │  ├─ On success: log metrics, continue
   │  └─ On failure: retry with backoff (max 3 retries)

4. Contract executes task
   ├─ Verify keeper.require_auth()
   ├─ Load task from storage
   ├─ Verify interval constraint
   ├─ Verify funds available
   ├─ Update task state (deduct bounty, increment counter)
   ├─ Invoke target contract function
   ├─ Transfer bounty to keeper
   └─ Emit aether_task_executed event

5. Keeper collects metrics
   ├─ Store execution record in SQLite
   └─ Update keeper's running balance

6. Frontend polling (optional)
   ├─ Every 30s, refresh task list
   └─ Update UI with new execution_count and remaining_funds
```

## State Management

### Contract State

```
Persistent Storage:
  DataKey::Task(u64) → TaskConfig
  - creator: Address
  - target_contract: Address
  - function: Symbol
  - args: Vec<Val>
  - interval: u64
  - last_executed_at: u64
  - bounty_per_exec: i128
  - remaining_funds: i128
  - token: Address
  - is_active: bool
  - max_executions: u64
  - execution_count: u64

Instance Storage:
  DataKey::Admin → Address (single admin for contract)
  DataKey::TaskCounter → u64 (next task ID)
```

### Frontend State (Zustand Store)

```
wallet.ts:
  - pubKey: string | null
  - isConnecting: boolean
  - connect(): Promise<void>
  - disconnect(): void

tasks.ts:
  - tasks: Task[]
  - loading: boolean
  - error: string | null
  - fetchTasks(): Promise<void>
  - createTask(...): Promise<void>
  - pauseTask(id: u64): Promise<void>
  - resumeTask(id: u64): Promise<void>
```

### Keeper Metrics State (SQLite)

```
executions Table:
  - id (auto-increment)
  - task_id (u64)
  - executor_address (text)
  - executed_at (timestamp)
  - bounty_paid (i128)
  - status (enum: success, failed, retry)

summary Table:
  - keeper_address (text, primary key)
  - total_executed (u64)
  - total_succeeded (u64)
  - total_failed (u64)
  - total_earned (i128)
  - last_update (timestamp)
```

## Communication Protocols

### Keeper ↔ Blockchain (Soroban RPC)

**Protocol**: HTTP/JSON-RPC

**Endpoints**:
- `getAccount(pubkey: string) → Account` - Get account state
- `submitTransaction(tx: string) → TransactionResult` - Submit signed transaction
- `getTransaction(hash: string) → Transaction` - Poll for confirmation
- `invokeContract(contract_id, method, args) → ContractResult` - Read-only queries

**Example**: Query contract for tasks
```
Request:
POST /
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "invokeContract",
  "params": {
    "contract": "CAAAA...",
    "method": "get_task",
    "args": ["0"],
    "source_account": { "account_id": "GAAAA...", "sequence": "123" }
  }
}

Response:
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": { "xdr": "..." }
}
```

### Frontend ↔ Soroban RPC

**Library**: `@stellar/stellar-sdk` - High-level abstraction

**Pattern**: Build transaction → Sign with Freighter → Submit via RPC

```typescript
const server = new SorobanRpc.Server(RPC_URL);
const account = await server.getAccount(userAddress);
const tx = new TransactionBuilder(account, { ...options })
  .addOperation(operation)
  .setTimeout(300)
  .setNetworkPassphrase(Networks.TESTNET_NETWORK_PASSPHRASE)
  .build();
const signed = await signTransaction(tx.toXDR());
const result = await server.submitTransaction(signed);
```

### Events & Indexing

**On-chain Events**: Contract emits Soroban events for off-chain indexing

```rust
env.events().publish(
  (Symbol::new(env, "aether_task_registered"),),
  (task_id, creator, bounty_per_exec),
);
```

**Optional Indexer**: Parse events and populate database for efficient queries

```sql
tasks Table:
  - task_id (u64)
  - creator (address)
  - target_contract (address)
  - bounty_per_exec (i128)
  - registered_at (timestamp)

executions Table:
  - task_id (u64)
  - keeper (address)
  - executed_at (timestamp)
  - bounty_paid (i128)
```

## Deployment Topology

### Development

```
Local Machine:
├── Soroban CLI (local testnet)
├── Contract: Deploy to local Soroban instance
├── Keeper: Run locally, poll local contract
└── Frontend: npm run dev, connect to local contract
```

### Testnet

```
Stellar Testnet:
├── Contract: Deployed to testnet
├── Keeper Operators: Multiple nodes worldwide
│   └── Each runs keeper-cli against testnet RPC
├── Frontend: Hosted on CDN, connects to public testnet RPC
└── Indexer: Optional service parsing contract events
```

### Production (Future)

```
Stellar Mainnet:
├── Contract: Deployed to mainnet, immutable
├── Keeper Network: Large decentralized set of operators
│   └── Economic incentive to maintain uptime
├── Frontend: Hosted on decentralized CDN
├── Indexer: High-availability service for queries
└── Monitoring: Alert on contract anomalies
```

## Scalability Considerations

### Contract Layer

- **Storage**: Task count unbounded; linear storage cost per task
- **Computation**: O(1) task execution (constant-time validation and state update)
- **Events**: Events indexed off-chain; no on-chain overhead

### Keeper Layer

- **Polling**: O(n) where n = total tasks; can be optimized with indexer
- **Concurrent Execution**: Bounded by `max_concurrent_tasks` config (prevents DOS)
- **Retry Logic**: Exponential backoff prevents RPC flooding

### Frontend Layer

- **Task List**: Paginated queries reduce load
- **Concurrent Users**: Stateless HTTP; scales horizontally via CDN/load balancing
- **Wallet Integration**: Per-user session; no shared state

## Resilience & Failure Modes

### Keeper Unavailability

- Task executions delayed until keeper comes online
- No data loss (all state on-chain)
- Auto-resume on keeper restart (idempotent design)

### RPC Endpoint Failure

- Keeper switches to backup endpoint (operator-configured)
- Exponential backoff prevents cascade failures
- Frontend shows error; user retries manually

### Token Transfer Failure

- Transaction reverts entirely (no partial state update)
- Keeper must retry or investigate balance

### Malicious Target Contract

- Invocation wrapped in `try_invoke_contract` (no panic)
- Execution error logged; bounty refunded
- Keeper continues processing other tasks

## Monitoring & Observability

### Keeper Metrics

```
Logs (via tracing):
  TRACE: Polling for tasks
  DEBUG: Task ready: task_id=5
  INFO: Executing task 5
  WARN: Retry #2 for task 5
  ERROR: Task 5 execution failed: insufficient balance

Database (SQLite):
  SELECT task_id, status, COUNT(*) FROM executions GROUP BY task_id
  SELECT SUM(bounty_paid) FROM executions WHERE keeper_address = ?
```

### Contract Events

```
aether_task_registered (task_id, creator, bounty_per_exec)
aether_task_executed (task_id, keeper, bounty_paid)
aether_task_paused (task_id, reason)
aether_task_resumed (task_id)
aether_task_cancelled (task_id, reason)
aether_funds_deposited (task_id, amount, depositor)
aether_funds_withdrawn (task_id, amount, recipient)
aether_execution_error (task_id, error_code)
```

## References

- [Soroban Architecture](https://soroban.stellar.org/docs/learn/architecture)
- [Stellar Smart Contracts](https://soroban.stellar.org/docs)
- [Freighter Wallet API](https://docs.freighter.app)
