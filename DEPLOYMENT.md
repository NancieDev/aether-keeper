# Deployment Guide

## Overview

This guide covers deploying Aether Keeper to Soroban testnet and mainnet.

## Prerequisites

- Rust 1.70+ with `wasm32-unknown-unknown` target
- Soroban CLI: `cargo install soroban-cli`
- Node.js 18+ for frontend (optional)
- Funded Stellar account (testnet or mainnet)

## Testnet Deployment

### 1. Setup Testnet Account

```bash
# Create testnet identity
soroban config identity create admin --network testnet

# Fund with Friendbot (testnet only)
curl "https://friendbot.stellar.org/?addr=$(soroban config identity address admin)"

# Verify funding
soroban config identity balance admin --network testnet
```

### 2. Build Smart Contract

```bash
cd contracts/aether_core
cargo build --target wasm32-unknown-unknown --release

# Contract WASM is at: target/wasm32-unknown-unknown/release/aether_core.wasm
ls -lh target/wasm32-unknown-unknown/release/aether_core.wasm
```

### 3. Deploy Contract

```bash
# From project root
soroban contract deploy \
  --wasm contracts/aether_core/target/wasm32-unknown-unknown/release/aether_core.wasm \
  --source-account admin \
  --network testnet

# Output: Contract ID (CABC123...)
# Example: CABFKTLQJ4NBIASF7KZLQMWAFVOBZJ7HQNLKB57EKNGVS7A6HLLZLP
```

Save the contract ID for later:
```bash
export AETHER_CONTRACT_ID="CABF..."  # Replace with your contract ID
```

### 4. Initialize Contract

```bash
soroban contract invoke \
  --id $AETHER_CONTRACT_ID \
  --source-account admin \
  --network testnet \
  -- \
  initialize \
  --admin $(soroban config identity address admin)

# Output: {"ok":"initialized"}
```

### 5. Configure Keeper CLI

```bash
cd keeper-cli

# Create keeper account (funded testnet account)
soroban config identity create keeper --network testnet
curl "https://friendbot.stellar.org/?addr=$(soroban config identity address keeper)"

# Initialize keeper
cargo run -- init \
  --secret-key $(soroban config identity show-secret keeper) \
  --rpc-url https://soroban-testnet.stellar.org \
  --contract-id $AETHER_CONTRACT_ID

# Verify initialization
cargo run -- balance
```

### 6. Start Keeper Daemon

```bash
# Terminal 1: Start keeper polling and executing tasks
cd keeper-cli
cargo run -- start

# Output:
# 2024-01-15 10:00:00 INFO keeper starting
# 2024-01-15 10:00:05 INFO polling tasks: 0 active, contract_id=CAB...
```

### 7. Deploy Frontend (Optional)

```bash
cd frontend

# Create .env.local
cat > .env.local << EOF
NEXT_PUBLIC_SOROBAN_RPC_URL=https://soroban-testnet.stellar.org
NEXT_PUBLIC_NETWORK_ID=testnet
NEXT_PUBLIC_AETHER_CONTRACT_ID=$AETHER_CONTRACT_ID
EOF

# Install and build
npm install
npm run build

# Start dev server
npm run dev

# Visit http://localhost:3000
```

### 8. Test End-to-End

```bash
# In frontend, create a task via UI
# Or manually via CLI:

soroban contract invoke \
  --id $AETHER_CONTRACT_ID \
  --source-account admin \
  --network testnet \
  -- \
  register_task \
  --creator $(soroban config identity address admin) \
  --target-contract $(soroban config identity address admin) \
  --function-name test \
  --interval 60 \
  --bounty-per-exec 500000000 \
  --initial-funding 5000000000

# Task should appear in keeper logs
# Keeper executes when interval elapsed
```

---

## Mainnet Deployment

**⚠️ WARNING: Mainnet deployment requires real XLM and cannot be undone. Proceed with caution.**

### 1. Prepare Mainnet Account

```bash
# Create mainnet identity
soroban config identity create admin-mainnet --network public

# Fund account (requires external transfer to this address)
soroban config identity address admin-mainnet

# Verify funding
soroban config identity balance admin-mainnet --network public

# Minimum required: 10 XLM (for deployment + initialization + testing)
```

### 2. Deploy to Mainnet

```bash
# Build contract (same WASM as testnet)
cd contracts/aether_core
cargo build --target wasm32-unknown-unknown --release

# Deploy
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/aether_core.wasm \
  --source-account admin-mainnet \
  --network public

# Output: Contract ID (CAB...)
export AETHER_CONTRACT_ID_MAINNET="CAB..."
```

### 3. Initialize Mainnet Contract

```bash
soroban contract invoke \
  --id $AETHER_CONTRACT_ID_MAINNET \
  --source-account admin-mainnet \
  --network public \
  -- \
  initialize \
  --admin $(soroban config identity address admin-mainnet)
```

### 4. Configure Mainnet Keeper

```bash
# Create mainnet keeper account (funded)
soroban config identity create keeper-mainnet --network public

# Fund account with minimum 5 XLM
# Transfer XLM to keeper address manually

# Initialize keeper
cd keeper-cli
cargo run -- init \
  --secret-key $(soroban config identity show-secret keeper-mainnet) \
  --rpc-url https://soroban-mainnet.stellar.org \
  --contract-id $AETHER_CONTRACT_ID_MAINNET
```

### 5. Start Mainnet Keeper

```bash
# Start keeper daemon on mainnet
cd keeper-cli
cargo run -- start

# Monitor logs for execution
# WARNING: Each execution costs real XLM in gas
```

### 6. Deploy Frontend to Production

```bash
cd frontend

# Update .env.production
cat > .env.production.local << EOF
NEXT_PUBLIC_SOROBAN_RPC_URL=https://soroban-mainnet.stellar.org
NEXT_PUBLIC_NETWORK_ID=public
NEXT_PUBLIC_AETHER_CONTRACT_ID=$AETHER_CONTRACT_ID_MAINNET
EOF

# Build for production
npm run build

# Deploy to hosting (Vercel, Netlify, etc.)
# Example with Vercel:
# vercel --prod
```

---

## Deployment Verification

### Verify Smart Contract

```bash
# Check contract info
soroban contract info --id $AETHER_CONTRACT_ID --network testnet

# Get contract source (admin address)
soroban contract info --id $AETHER_CONTRACT_ID --network testnet | grep admin
```

### Verify Keeper

```bash
# Check keeper balance
keeper balance

# Check keeper config
keeper config show

# Test execution manually
keeper execute 1
```

### Verify Frontend

```bash
# Check environment variables are loaded
curl http://localhost:3000/api/config

# Test wallet connection
# Open http://localhost:3000
# Click "Connect Freighter"
# Sign transaction in Freighter
```

---

## Monitoring & Maintenance

### Keeper Metrics

```bash
# View execution metrics
keeper metrics

# View logs
keeper logs

# Check pending transactions
keeper pending
```

### Contract Events

```bash
# Monitor contract events
soroban contract events \
  --id $AETHER_CONTRACT_ID \
  --network testnet \
  --count 50

# Filter by event type
soroban contract events \
  --id $AETHER_CONTRACT_ID \
  --network testnet \
  --type task_executed
```

### Keeper Health

```bash
# Health check endpoint
keeper health

# Output:
# {
#   "status": "healthy",
#   "keeper_address": "GAB...",
#   "balance": 5000000000,
#   "tasks_executed": 23,
#   "last_execution": "2024-01-15T10:15:00Z"
# }
```

---

## Rollback & Recovery

### Pause All Tasks

If issues arise, pause all tasks to prevent further execution:

```bash
# Keeper CLI pause command
keeper pause-all

# Or manually via contract
soroban contract invoke \
  --id $AETHER_CONTRACT_ID \
  --source-account admin \
  --network testnet \
  -- \
  pause_all
```

### Restart Keeper

```bash
# Stop keeper daemon (Ctrl+C)
# Restart with fresh state
cd keeper-cli
cargo run -- start
```

### Restore from Backup

```bash
# Keeper stores state in ~/.aether/keeper-config.json
# Backup before deployment:
cp ~/.aether/keeper-config.json ~/.aether/keeper-config.json.backup

# Restore if needed
cp ~/.aether/keeper-config.json.backup ~/.aether/keeper-config.json
```

---

## Troubleshooting

### "Contract not found"
```bash
# Verify contract ID
echo $AETHER_CONTRACT_ID

# Check contract exists
soroban contract info --id $AETHER_CONTRACT_ID --network testnet
```

### "Insufficient balance"
```bash
# Fund account
curl "https://friendbot.stellar.org/?addr=$(soroban config identity address admin)"

# Check balance
soroban config identity balance admin --network testnet
```

### "Transaction timeout"
```bash
# Check RPC connectivity
curl https://soroban-testnet.stellar.org/health

# Increase timeout in keeper config
keeper config set-timeout 30
```

### "Freighter not connecting"
```bash
# Ensure Freighter extension installed
# Check browser console for errors
# Try clearing Freighter cache and reconnecting
```

---

## Security Checklist

Before mainnet deployment:

- [ ] Contract code reviewed by security team
- [ ] Keeper secrets encrypted (AES-256-GCM)
- [ ] RPC endpoints validated (HTTPS only)
- [ ] Rate limiting configured
- [ ] Monitoring alerts setup
- [ ] Incident response plan documented
- [ ] Backup procedures tested
- [ ] Rollback procedures tested

---

## Support

- **Deployment Issues**: [GitHub Issues](https://github.com/NancieDev/aether-keeper/issues)
- **Documentation**: [README.md](./README.md), [ARCHITECTURE.md](./ARCHITECTURE.md)
- **Quick Reference**: [QUICKSTART.md](./QUICKSTART.md)
