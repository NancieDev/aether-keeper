# Quick Start - Aether Keeper

## For Operators (Running Keepers)

### Prerequisites
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install soroban-cli
```

### Setup & Run
```bash
# Clone repo
git clone https://github.com/NancieDev/aether-keeper
cd aether-keeper

# Install dependencies
make setup

# Initialize keeper
cd keeper-cli
cargo run -- init \
  --secret-key SABC... \
  --rpc-url https://soroban-testnet.stellar.org \
  --contract-id CAAA...

# Start keeper daemon (polls and executes tasks)
cargo run -- start

# Check balance
cargo run -- balance

# List active tasks
cargo run -- list --status active
```

**Result**: Keeper daemon runs indefinitely, polling contract every 60s, executing ready tasks.

---

## For Task Creators (Using Frontend)

### Prerequisites
- Install [Freighter Wallet](https://www.freighter.app/)
- Have testnet XLM funded (~50 XLM for tasks)

### Create & Manage Tasks
1. Visit frontend: `http://localhost:3000` (or deployment URL)
2. Click "Connect Freighter Wallet"
3. Click "+ Create Task"
4. Fill in:
   - **Target Contract**: `C...` (contract to invoke)
   - **Function Name**: `compound_yield` (function to call)
   - **Interval**: `86400` (seconds, e.g., 1 day)
   - **Bounty**: `10` XLM (payment per keeper execution)
   - **Funding**: `100` XLM (total bounty pool)
5. Click "Create Task"
6. Freighter prompts for signature
7. Task appears in dashboard

**Dashboard Shows**:
- ✅ All your tasks (active, paused, completed)
- 📊 Task statistics (execution count, remaining funds)
- ⏸️ Controls: pause/resume, deposit/withdraw, details

---

## For Developers (Contributing Code)

### Setup Development Environment
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Soroban
cargo install soroban-cli

# Install Node.js 18+
node --version  # Should be 18+

# Clone & setup
git clone https://github.com/NancieDev/aether-keeper
cd aether-keeper
make setup
```

### Development Workflow
```bash
# Create branch
git checkout -b feat/your-feature

# Make changes
# Test
make test

# Lint
make lint

# Format
make format

# Commit with clear message
git commit -m "feat(contract): add max_executions validation"

# Push & create PR
git push origin feat/your-feature
```

**Code Standards**:
- Format: `cargo fmt` (Rust), `npm run format` (TypeScript)
- Lint: `cargo clippy` (Rust), `npm run lint` (TypeScript)
- Test: `make test` (all), `make test-contract` (contract only)
- Documentation: Add doc comments to public functions

**Good First Issues**: Look for `good-first-issue` label

---

## For Deployers (Testnet/Mainnet)

### Testnet Deployment
```bash
# Create testnet identity
soroban config identity create admin --network testnet

# Fund with Friendbot (testnet only)
curl "https://friendbot.stellar.org/?addr=$(soroban config identity address admin)"

# Deploy
make deploy ADMIN_KEY=SABC... NETWORK=testnet

# Outputs: Contract ID
```

### Production (Mainnet)
```bash
# Use hardware wallet (recommended)
soroban config identity create admin-hw --network public

# Deploy with caution
make deploy-prod ADMIN_KEY=... NETWORK=public
```

See **[DEPLOYMENT.md](./DEPLOYMENT.md)** for detailed steps.

---

## Architecture Overview

```
┌─────────────────────┐
│  Task Creators      │ (Create tasks via frontend)
│  (Web UI / Freighter)
└──────────┬──────────┘
           │ register_task()
           ▼
   ┌──────────────────┐
   │  Smart Contract  │ (Soroban)
   │  AetherKeeper    │
   └────────┬─────────┘
            │ Events: task_registered, task_executed
            │
     ┌──────▼──────┐
     │    Events   │ (Off-chain indexing)
     │   Listener  │
     └─────────────┘
            │
     ┌──────▼─────────────┐
     │  Keeper Network    │
     │  (Off-chain)       │
     │  ┌──────────────┐  │
     │  │  Keeper CLI  │  │ (Polls → Executes → Earns bounties)
     │  │  Daemon (1+) │  │
     │  └──────────────┘  │
     └────────────────────┘
```

**Flow**:
1. Task creator registers task with bounty
2. Keeper daemon polls contract every 60s
3. Keeper finds executable tasks (interval met, funds available)
4. Keeper executes target contract function
5. Keeper earns bounty
6. Task completes or repeats

---

## Key Files

| File | Purpose |
|------|---------|
| **README.md** | Project overview |
| **ARCHITECTURE.md** | System design & data flow |
| **DEPLOYMENT.md** | How to deploy (testnet & mainnet) |
| **SECURITY.md** | Security considerations & threat model |
| **CONTRIBUTING.md** | How to develop & contribute |
| **contracts/aether_core/src/lib.rs** | Smart contract implementation |
| **keeper-cli/src/executor.rs** | Keeper polling & execution logic |
| **frontend/src/** | Web UI (Next.js + React) |
| **scripts/deploy.sh** | Deployment automation |
| **Makefile** | Build commands |

---

## Common Commands

```bash
# Build everything
make build

# Run all tests
make test

# Format code
make format

# Lint code
make lint

# Generate docs
make docs

# Deploy to testnet
make deploy ADMIN_KEY=SABC...

# Start keeper daemon
cd keeper-cli && cargo run -- start

# Start frontend dev server
cd frontend && npm run dev
```

---

## Troubleshooting

### "Contract not found"
```bash
# Verify contract ID
echo $NEXT_PUBLIC_AETHER_CONTRACT_ID

# Check contract exists
soroban contract info --id $CONTRACT_ID --network testnet
```

### "Insufficient funds"
```bash
# Check keeper balance
keeper-cli balance

# Fund keeper account (testnet)
curl "https://friendbot.stellar.org/?addr=<keeper-address>"
```

### "Transaction timeout"
```bash
# Check RPC connectivity
curl https://soroban-testnet.stellar.org/health

# Increase timeout in executor config
```

### "Freighter not signing"
```bash
# Ensure Freighter extension installed
# Try re-connecting wallet
# Check browser console for errors
```

---

## Next Steps

1. **Read** [ARCHITECTURE.md](./ARCHITECTURE.md) to understand the system
2. **Deploy** to testnet following [DEPLOYMENT.md](./DEPLOYMENT.md)
3. **Run** keeper daemon to execute tasks
4. **Create** tasks via frontend
5. **Monitor** execution and earnings
6. **Contribute** - see [CONTRIBUTING.md](./CONTRIBUTING.md)

---

## Support

- **Questions**: Check [ARCHITECTURE.md](./ARCHITECTURE.md)
- **Deployment Issues**: See [DEPLOYMENT.md](./DEPLOYMENT.md)
- **Security**: Read [SECURITY.md](./SECURITY.md)
- **Contributing**: See [CONTRIBUTING.md](./CONTRIBUTING.md)
- **Bugs**: Open GitHub issue

---

**Ready to automate? Let's go! 🚀**
