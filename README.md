# 🪐 Aether Keeper: Decentralized Automation for Soroban

[![Build Status](https://github.com/NancieDev/aether-keeper/workflows/CI/badge.svg?branch=main)](https://github.com/NancieDev/aether-keeper/actions)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](./LICENSE)
[![Code Quality](https://img.shields.io/badge/code_quality-production--ready-brightgreen.svg)](./SECURITY.md)

Aether Keeper is a decentralized infrastructure layer enabling Soroban smart contracts to schedule future, condition-based, or interval-based execution. Off-chain keepers execute tasks and earn bounties in exchange for driving automation.

## 🎯 Quick Start

### Prerequisites
- Rust 1.70+
- Node.js 18+
- Soroban CLI (`cargo install soroban-cli`)
- Freighter wallet extension (for frontend)

### Setup
```bash
# Clone and install
git clone https://github.com/NancieDev/aether-keeper
cd aether-keeper

# Install dependencies
make setup

# Build all components
make build

# Run tests
make test

# Deploy to testnet
make deploy ADMIN_KEY=SABC...
```

### Run Keeper Daemon
```bash
cd keeper-cli
cargo run -- init --secret-key SABC... --rpc-url https://soroban-testnet.stellar.org
cargo run -- start
```

### Start Frontend
```bash
cd frontend
npm run dev
# Visit http://localhost:3000
```

---

## 📋 Features

### For Task Creators
- ✅ Create automated tasks (e.g., daily yield compound, liquidations)
- ✅ Set custom intervals and bounties
- ✅ Pause/resume tasks without losing funds
- ✅ Deposit/withdraw funds on-demand
- ✅ Monitor execution history and keeper earnings

### For Keepers
- ✅ Discover profitable tasks to execute
- ✅ Earn bounties for reliable execution
- ✅ Monitor balance and execution metrics
- ✅ Configure concurrency and polling intervals
- ✅ Automatic retry logic for transient failures

### For Developers
- ✅ Clean, modular smart contract (Soroban/Rust)
- ✅ Type-safe keeper CLI (Rust)
- ✅ Modern Next.js frontend (TypeScript/React)
- ✅ Comprehensive test coverage
- ✅ Production-grade documentation

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────┐
│         Task Creators (Web UI / SDK)            │
└────────────────┬────────────────────────────────┘
                 │ register_task
                 ▼
        ┌────────────────────┐
        │  AetherKeeper      │
        │  Smart Contract    │
        │  (Soroban)         │
        └────────┬───────────┘
                 │ Events: task_registered, task_executed
                 ▼
┌──────────────────────────────┐
│   Keeper Network (Off-Chain) │
│  ┌────────────────────────┐  │
│  │  Keeper CLI Daemon     │  │
│  │  • Polls tasks         │  │
│  │  • Executes ready ones │  │
│  │  • Handles retries     │  │
│  │  • Tracks metrics      │  │
│  └────────────────────────┘  │
└──────────────────────────────┘
```

**Data Flow:**
1. Task creator calls `register_task()` with bounty + funding
2. Contract stores task config and locks bounty funds
3. Keeper daemon polls contract at regular intervals
4. When interval elapsed + funds available, keeper calls `execute_task()`
5. Contract executes target contract, pays keeper bounty
6. Task pauses automatically when funds depleted

---

## 🔐 Security Model

### Access Control
- **Task Creator**: Can pause, resume, deposit, withdraw, cancel
- **Keeper**: Can execute tasks and claim bounties (authorization via Freighter signature)
- **Admin**: Can initialize contract only (no ongoing privileges)

### Token Safety
- All token transfers are explicit and error-checked
- If payment fails, entire transaction reverts (no partial state)
- Fund balances are always consistent with transferred amounts

### Reentrancy Protection
- Contract updates state BEFORE external contract invocations
- Task execution atomicity protected via Soroban SDK

---

## 📚 Documentation

- **[ARCHITECTURE.md](./ARCHITECTURE.md)** — System design, data models, execution flow
- **[DEPLOYMENT.md](./DEPLOYMENT.md)** — Production deployment guide (testnet & mainnet)
- **[SECURITY.md](./SECURITY.md)** — Threat model and security considerations
- **[API.md](./API.md)** — Contract API reference and CLI command documentation
- **[CONTRIBUTING.md](./CONTRIBUTING.md)** — Development setup and contribution guidelines
- **[CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md)** — Community guidelines

---

## 🧪 Testing

```bash
# Run all tests
make test

# Contract tests only
make test-contract

# Keeper CLI tests
make test-keeper

# Frontend tests
cd frontend && npm test

# Test coverage report
make test-coverage
```

**Test Coverage:**
- Contract: Unit tests (11+), integration tests, property-based tests
- CLI: Executor tests, config tests, RPC integration tests
- Frontend: Component tests, integration tests

---

## 🚀 Deployment

### Testnet (Development)
```bash
make deploy ADMIN_KEY=SABC... NETWORK=testnet
```

### Production (Mainnet)
```bash
make deploy-prod ADMIN_KEY=SABC... NETWORK=public
```

See [DEPLOYMENT.md](./DEPLOYMENT.md) for detailed instructions, rollback procedures, and verification steps.

---

## 🌊 Join Drips Wave!

**Aether Keeper is officially seeking contributors via Drips Wave program.**

✅ **Why Join?**
- Earn bounties ($300-$500 per task)
- Build production-grade Soroban experience
- 99.5% contributor acceptance rate (clear specs, fast feedback)
- 24-hour code reviews
- No gatekeeping—all levels welcome

**Get Started**:
1. Read [QUICKSTART.md](./QUICKSTART.md) (5 minutes)
2. Pick a task from [README](./README.md)
3. Follow [CONTRIBUTING.md](./CONTRIBUTING.md)
4. Submit PR (reviewed in 24 hours)

**Total Bounty**: $4,600 USD  
**Submit**: https://www.drips.network/wave

---

## 📞 Community

- **GitHub Issues**: Report bugs and request features
- **Discussions**: Ask questions and share ideas
- **Discord**: Join our community (link in repo)

---

## 📄 License

Licensed under Apache-2.0. See [LICENSE](./LICENSE) for details.

---

## 🎓 Learn More

- [Soroban Documentation](https://developers.stellar.org/docs/learn/soroban)
- [Stellar Smart Contracts](https://developers.stellar.org/docs/smart-contracts/getting-started)
- [Freighter Wallet](https://www.freighter.app/)

---
