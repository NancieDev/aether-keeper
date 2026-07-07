# Security Considerations & Best Practices

## Contract Security

### Access Control
✅ **Implemented**
- All state-mutating functions require `require_auth()` from the caller
- Only task creators can pause, resume, deposit, withdraw, or cancel tasks
- Only the initialization caller can initialize the contract (prevents re-initialization attacks)

### Token Transfer Safety
✅ **Implemented**
- All token transfers use `.map_err(|_| ContractError::TransferFailed)?`
- If a transfer fails, the entire transaction reverts (atomic behavior)
- No partial state updates on transfer failure
- Bounty deductions and payments are checked before execution

### Reentrancy Protection
✅ **Implemented**
- Contract updates state BEFORE invoking external contracts
- Execution sequence: validate → update state → invoke target → pay keeper
- This prevents reentrancy attacks where target contract tries to manipulate task state

### Arithmetic Safety
✅ **Implemented via Soroban SDK**
- i128 arithmetic operations are checked for overflow/underflow
- Soroban SDK panics (reverts transaction) on overflow
- All bounty calculations are safe within i128 bounds

### Input Validation
✅ **Implemented**
- `interval > 0` (prevents instant re-execution)
- `bounty_per_exec > 0` (prevents zero-payment exploitation)
- `initial_funding >= bounty_per_exec` (prevents underfunded tasks)
- Task existence checked before operations (prevents orphan states)

### Fund Custody
✅ **Implemented**
- All deposited funds are held in the contract's account
- Only the contract can approve disbursements via token transfers
- Creator can only withdraw up to remaining_funds (no over-withdrawal)
- Auto-pause on fund depletion prevents stuck tasks

---

## Keeper Security

### Private Key Management
⚠️ **CURRENT: Plaintext Storage**

**Issue**: Secret keys stored in `~/.aether/keeper-config.json` are unencrypted.

**Risk**: If attacker gains filesystem access, they can extract all keeper secrets.

**Recommendations**:
1. **Encrypt at rest** (AES-256-GCM):
   ```rust
   // Use argon2 for key derivation from passphrase
   let key = argon2::hash_raw(passphrase, salt, config)?;
   let secret = encrypt_aes_256_gcm(secret_key_string, key)?;
   ```

2. **Support hardware wallets** (e.g., Ledger):
   ```rust
   // Delegate signing to hardware wallet instead of storing secret
   let signature = ledger.sign_transaction(tx)?;
   ```

3. **Implement key rotation**:
   - Periodically generate new keeper identity
   - Migrate to new key via multi-sig or gradual transition
   - Revoke old key in keeper marketplace (future)

### RPC Endpoint Security
⚠️ **ASSUMES HTTPS**

**Current**: Code assumes `rpc_url` is HTTPS and trustworthy.

**Risk**: Man-in-the-middle attacks if RPC endpoint is compromised.

**Recommendations**:
1. **Validate RPC certificate**:
   ```rust
   let client = reqwest::Client::builder()
       .danger_accept_invalid_certs(false)
       .build()?;
   ```

2. **Use official Stellar RPC endpoints**:
   - Testnet: `https://soroban-testnet.stellar.org`
   - Mainnet: `https://mainnet.soroban.stellar.org`

3. **Monitor RPC responses** for suspicious patterns

### Transaction Security
✅ **Implemented**
- Keeper signs all execute_task transactions with private key
- Keeper address must match on-chain expectations
- RPC validates transaction signatures before acceptance
- Replay attacks prevented by Soroban nonce mechanism

### Concurrent Execution
✅ **Implemented**
- Multiple keepers can execute different tasks in parallel
- Each keeper polls independently (no coordination needed)
- Task state prevents double-execution (interval + last_executed_at checks)

---

## Frontend Security

### Wallet Integration
✅ **Freighter Handles**
- Freighter wallet manages private keys (never exposed to frontend)
- User explicitly approves each transaction (no silent approvals)
- Frontend only has access to public key (pubKey)

### Input Validation
✅ **Should Be Implemented**
- Validate contract address format before submission
- Validate numeric fields (bounty, interval) before transaction building
- Validate function names don't contain dangerous characters
- Validate contract ABI matches expected function signatures

### Local Storage Security
⚠️ **Avoid Storing Sensitive Data**

**Current**: Pubkey stored in browser localStorage (safe).

**Risk**: Never store private keys, secret keys, or sensitive task data in localStorage.

**Recommendations**:
1. Store only pubKey and network in localStorage
2. Fetch task list from contract on every page load (don't cache)
3. Use sessionStorage for temporary TX hashes (clear on logout)
4. Never store token balances if user account can be compromised

### Transaction Parameter Validation
⚠️ **TODO**

Add type checking before contract invocation:
```typescript
// Validate args before transaction building
if (!isValidAddress(targetContract)) throw new Error("Invalid contract address");
if (interval < 60) throw new Error("Interval must be >= 60 seconds");
if (bounty <= 0) throw new Error("Bounty must be > 0");
if (funding < bounty) throw new Error("Funding must be >= bounty");
```

---

## Network Security

### Soroban Consensus Assumptions
✅ **Leverages Stellar Foundation**
- Soroban RPC runs on Stellar consensus (Byzantine Fault Tolerant)
- Transactions are cryptographically signed
- Ledger state is replicated and validated by validators

### Fee Market
⚠️ **Not Implemented**

Current: No mechanism to adapt to changing Soroban fees.

**Risk**: If fees spike, keepers might not cover TX costs; if fees are minimal, spam attacks become cheap.

**Recommendations**:
1. Monitor average block fees
2. Adjust bounty_per_exec based on estimated TX cost (bounty >= 1.5 * estimated_fee)
3. Implement fee-aware keeper scheduling (execute high-bounty tasks first during congestion)

### Ledger Finality
✅ **Guaranteed by Stellar**
- All committed transactions are final (no forks)
- Task execution is immutable once confirmed

---

## Attack Vectors & Mitigations

### 1. Task Registration DOS
**Attack**: Attacker registers millions of tasks with minimal funding.

**Impact**: Keepers waste RPC queries scanning junk tasks.

**Mitigation**:
- Increase minimum initial_funding (e.g., >= 1 XLM)
- Implement task registration fee (small amount burned or sent to treasury)
- Rate-limit tasks per creator (e.g., max 100 active tasks per address)

### 2. Bounty Theft via Malicious Target Contract
**Attack**: Task creator registers task pointing to malicious contract that reverts and steals bounty.

**Impact**: Keeper executes task, malicious contract triggers, keeper still gets paid but task fails.

**Mitigation**:
- Task creator should test target contract before registration
- Keepers should inspect target contract code before executing (off-chain)
- Implement task reputation score (number of successful executions)

### 3. Keeper Bankruptcy
**Attack**: Keeper runs out of XLM for TX fees, stops executing tasks.

**Impact**: High-fee periods become unserviceable.

**Mitigation**:
- Bounty_per_exec should exceed estimated TX fee by 50%+
- Keepers should maintain > 5 XLM buffer for TX fees (not bounty-funded)
- Implement keeper balance monitoring with alerts

### 4. Infinite Execution Loop
**Attack**: Task creator registers task targeting their own contract, which calls execute_task() recursively.

**Impact**: Unbounded gas consumption, potential contract failure.

**Mitigation**:
- max_executions field limits total executions
- Set conservative default (e.g., 365 for daily tasks = 1 year)
- Creator can set max_executions explicitly during registration

### 5. Private Key Leakage
**Attack**: Keeper config file is compromised, attacker extracts secret key.

**Impact**: Attacker can execute tasks and collect bounties using keeper's account.

**Mitigation**:
- Encrypt keeper secrets at rest (AES-256)
- Use hardware wallet for key storage
- Monitor keeper account for unauthorized activity
- Rotate keys if compromise suspected

### 6. RPC Endpoint Takeover
**Attack**: Attacker intercepts HTTPS traffic to RPC endpoint or DNS hijacking.

**Impact**: Attacker can forge transaction confirmations, preventing task execution or creating false executions.

**Mitigation**:
- Use official Stellar Foundation RPC endpoints
- Verify certificate pinning for critical endpoints
- Monitor for unusual response patterns
- Use multiple RPC endpoints for fallback

---

## Monitoring & Incident Response

### Alerts to Set Up

| Alert | Threshold | Action |
|-------|-----------|--------|
| High token transfer failures | > 10% | Investigate RPC/contract issues |
| Keeper balance low | < 1 XLM | Top up keeper account immediately |
| Task execution lag | > 5 min | Check keeper daemon status |
| Unusual task registration | > 100 tasks/day | Investigate for DOS |
| Authorization failures | > 5% | Check keeper key setup |

### Incident Response Playbook

**Scenario: Keeper Private Key Compromised**
1. Identify compromised address
2. Stop keeper daemon immediately
3. Generate new keeper identity
4. Update contract/config with new keeper address
5. Monitor old address for unauthorized activity
6. Audit all transactions from old key

**Scenario: RPC Endpoint Unresponsive**
1. Keeper daemon will auto-retry with exponential backoff
2. After 3 retries, keeper logs warning but continues polling
3. Check RPC status: `curl https://soroban-testnet.stellar.org/health`
4. If RPC is down, keepers will resume when RPC recovers
5. Monitor for backlog of unexecuted tasks

**Scenario: Contract Initialization Failed**
1. Check error logs for failure reason
2. Verify admin address and secret key match
3. Verify contract exists on network: `soroban contract info --id <CONTRACT_ID>`
4. Retry initialization: `soroban contract invoke --id <CONTRACT_ID> -- initialize --admin <ADMIN>`
5. If still fails, redeploy fresh contract

---

## Compliance & Standards

### Token Standards
- Soroban token contracts follow Stellar Asset Anchor framework
- XLM (native Stellar token) is supported
- Other token contracts supported if they implement standard transfer interface

### Smart Contract Best Practices
✅ Follows OWASP Smart Contract Top 10
- ✅ Reentrancy protection (state before external calls)
- ✅ Arithmetic safety (Soroban SDK handles overflow)
- ✅ Access control (require_auth on all mutations)
- ✅ Input validation (bounds checking on all parameters)
- ✅ Gas limits respected (Soroban enforces per transaction)

---

## Future Security Enhancements

1. **Multi-Sig Keepers**: Multiple keepers vote to execute high-value tasks
2. **Slashing Mechanism**: Misbehaving keepers lose staked amount
3. **Task Auditing**: Off-chain service audits and certifies target contracts
4. **Emergency Pause**: Admin can pause all tasks in emergency (e.g., bug discovered)
5. **Upgrade Mechanism**: Contract upgradability with governance (requires new deployment for now)

---

## Security Audit Checklist

- [ ] All token transfers have error handling (map_err)
- [ ] All state mutations require require_auth()
- [ ] State updated before external calls (reentrancy safe)
- [ ] Arithmetic checked for overflow (i128 bounds)
- [ ] Input validation on all public functions
- [ ] Events emitted for all significant state changes
- [ ] Keeper key encryption enabled
- [ ] RPC endpoints pinned/verified
- [ ] Frontend input validation implemented
- [ ] Error messages don't leak sensitive information
- [ ] Monitoring and alerting configured
- [ ] Incident response procedures documented

