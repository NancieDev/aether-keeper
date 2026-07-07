# Contributing to Aether Keeper

We welcome contributions from developers, designers, and documentation writers. This guide will help you get started.

## Development Setup

### Prerequisites
- Rust 1.70+ (`rustup update`)
- Node.js 18+
- Soroban CLI (`cargo install soroban-cli`)
- Git

### Clone & Setup
```bash
git clone https://github.com/NancieDev/aether-keeper
cd aether-keeper

# Install all dependencies
make setup

# Build everything
make build

# Run tests
make test
```

### Create Development Identity
```bash
soroban config identity create dev-user --network testnet
soroban config identity fund dev-user --network testnet
```

---

## Project Structure

```
aether-keeper/
├── contracts/aether_core/        # Smart contract (Rust/Soroban)
│   ├── src/lib.rs               # Main contract logic
│   ├── src/storage.rs           # Data structures
│   ├── src/events.rs            # Event definitions
│   └── src/tests.rs             # Contract tests
├── keeper-cli/                  # Keeper CLI daemon (Rust)
│   ├── src/main.rs              # CLI entry point
│   ├── src/executor.rs          # Task executor
│   ├── src/config.rs            # Configuration
│   └── src/error.rs             # Error types
├── frontend/                    # Web UI (Next.js/TypeScript)
│   ├── src/app/                 # App pages
│   ├── src/components/          # React components
│   └── src/hooks/               # Custom hooks
├── scripts/                     # Deployment & utility scripts
├── ARCHITECTURE.md              # System design
├── DEPLOYMENT.md                # Production deployment guide
├── SECURITY.md                  # Security considerations
├── Makefile                     # Build automation
└── README.md                    # Project overview
```

---

## Development Workflow

### 1. Pick an Issue
- Look for issues labeled `good-first-issue` if you're new
- Comment on the issue to claim it
- Ask questions if requirements are unclear

### 2. Create a Branch
```bash
git checkout -b feat/issue-number-short-name
# or
git checkout -b fix/issue-number-short-name
```

**Branch naming**:
- `feat/...` — New feature
- `fix/...` — Bug fix
- `docs/...` — Documentation
- `test/...` — Tests only

### 3. Make Changes

#### Smart Contract Changes
```bash
cd contracts/aether_core

# Test your changes
cargo test -- --nocapture

# Format code
cargo fmt

# Lint
cargo clippy --all-targets -- -D warnings
```

#### Keeper CLI Changes
```bash
cd keeper-cli

# Test your changes
cargo test

# Build binary
cargo build --release

# Run CLI
./target/release/keeper-cli --help
```

#### Frontend Changes
```bash
cd frontend

# Install dependencies
npm install

# Run dev server
npm run dev

# Format code
npm run format

# Lint
npm run lint

# Type check
npm run type-check

# Run tests
npm test
```

### 4. Commit Changes

Use descriptive commit messages:
```bash
git commit -m "feat(contract): add max_executions validation

- Prevents infinite execution loops
- Validates max_executions > 0 during registration
- Auto-pauses when limit reached
- Fixes #123"
```

**Commit guidelines**:
- Use present tense ("add" not "added")
- Reference issue numbers (#123)
- Include what and why, not how
- Keep commits focused on single concern

### 5. Push & Create PR
```bash
git push origin feat/issue-number-short-name
```

Then create a Pull Request on GitHub with:
- **Title**: Brief description of change
- **Description**: Why this change is needed, how it works
- **Fixes**: Link to related issue (#123)
- **Checklist**: ✅ Tests pass, ✅ Linting passes, ✅ Code reviewed

---

## Code Standards

### Rust
- Format with `cargo fmt`
- Lint with `cargo clippy -- -D warnings`
- Document public functions with doc comments:
  ```rust
  /// Register a new automation task
  ///
  /// # Arguments
  /// * `creator` - Address creating the task
  /// * `initial_funding` - Initial bounty pool
  ///
  /// # Errors
  /// Returns InvalidBounty if bounty <= 0
  pub fn register_task(...) -> Result<u64, ContractError> { ... }
  ```
- Write tests for all public functions and edge cases

### TypeScript/React
- Use strict TypeScript (`noImplicitAny`, `strictNullChecks`, etc.)
- Format with `prettier`
- Lint with `eslint`
- Component example:
  ```typescript
  interface Props {
    taskId: number;
    onExecute: (taskId: number) => Promise<void>;
  }

  export const TaskButton: React.FC<Props> = ({ taskId, onExecute }) => {
    // Component logic
  };
  ```
- Write component tests using React Testing Library

### Markdown Documentation
- Use clear, concise language
- Include code examples
- Add table of contents for long docs
- Keep line length <= 100 characters

---

## Testing

### Smart Contract Tests
```bash
cd contracts/aether_core
cargo test -- --nocapture --test-threads=1

# With coverage
cargo tarpaulin --out Html --output-dir coverage
```

**Test requirements**:
- Happy path tests (successful execution)
- Error path tests (invalid inputs)
- Edge case tests (boundary values)
- State consistency tests (invariants hold after operations)

Example test:
```rust
#[test]
fn test_register_task_insufficient_funding() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup
    let creator = Address::generate(&env);
    let token = env.register_stellar_asset_contract(/* ... */);

    // Create keeper contract
    let contract_id = env.register_contract(None, AetherKeeper);
    let client = AetherKeeperClient::new(&env, &contract_id);

    // Test: Register with bounty > funding should fail
    let result = client.try_register_task(
        &creator,
        &Address::generate(&env),
        &Symbol::new(&env, "test"),
        &vec![&env],
        60,
        100,  // bounty
        50,   // funding < bounty
        &token,
    );

    assert!(result.is_err());
}
```

### Frontend Tests
```bash
cd frontend
npm test
npm test -- --coverage
```

**Test requirements**:
- Component renders correctly
- User interactions work (clicks, form submissions)
- API calls are mocked and handled
- Error states display properly

Example test:
```typescript
import { render, screen, fireEvent } from '@testing-library/react';
import CreateTaskModal from '@/components/CreateTaskModal';

test('shows error on invalid contract address', () => {
  const onClose = jest.fn();
  render(<CreateTaskModal onClose={onClose} />);

  const input = screen.getByPlaceholderText(/contract/i);
  fireEvent.change(input, { target: { value: 'invalid' } });

  const button = screen.getByText(/Create/i);
  fireEvent.click(button);

  expect(screen.getByText(/invalid address/i)).toBeInTheDocument();
});
```

### Integration Tests
```bash
# Deploy contract to local Soroban + test end-to-end
make test-integration
```

---

## Documentation

### Adding Documentation
1. Write markdown in `/docs` or relevant `.md` file
2. Use clear headers and sections
3. Include code examples
4. Update README.md table of contents

### API Documentation
- Document all contract public functions
- Document all CLI commands
- Document all React component props

Example:
```markdown
### `register_task`

Creates a new automation task.

**Parameters:**
- `creator: Address` — Account funding the task
- `target_contract: Address` — Contract to invoke
- `function: Symbol` — Function name (e.g., "compound")
- `interval: u64` — Seconds between executions (must be > 0)
- `bounty_per_exec: i128` — Payment per execution (must be > 0)
- `initial_funding: i128` — Total bounty pool (must be >= bounty)
- `token: Address` — Token address for payments

**Returns:** `u64` — Task ID

**Errors:**
- `InvalidInterval` — interval == 0
- `InvalidBounty` — bounty <= 0
- `InsufficientFunding` — funding < bounty
- `TransferFailed` — Token transfer failed

**Example:**
```rust
let task_id = client.register_task(
    &creator,
    &target_contract,
    &Symbol::new(&env, "compound_yield"),
    &args,
    86400,  // 1 day
    10,     // 10 stroops
    1000,   // 1000 stroops
    &token,
)?;
```
```

---

## Pull Request Review Process

### Before Submitting
- [ ] All tests pass (`make test`)
- [ ] Code is formatted (`cargo fmt`, `npm run format`)
- [ ] Linting passes (`cargo clippy`, `npm run lint`)
- [ ] No console.log/debug statements left
- [ ] Documentation is updated
- [ ] Commit message is descriptive

### Review Checklist
Maintainers will check:
- ✅ Code quality and style
- ✅ Test coverage (>70%)
- ✅ No security vulnerabilities
- ✅ Documentation is clear
- ✅ No breaking changes (or justified)
- ✅ Performance impact assessed

---

## Issue Labels

| Label | Meaning |
|-------|---------|
| `good-first-issue` | Great for new contributors |
| `help-wanted` | Extra attention needed |
| `bug` | Something is broken |
| `enhancement` | New feature |
| `documentation` | Docs need improvement |
| `security` | Security concern |
| `performance` | Performance improvement |
| `blocked` | Waiting on something else |

---

## Communication

- **Issues**: Use GitHub issues for bug reports and feature requests
- **Discussions**: Ask questions in GitHub Discussions
- **Discord**: Join our community for real-time chat
- **Code Review**: Be respectful and constructive in PR reviews

---

## Recognition

We recognize all contributors in:
- CONTRIBUTORS.md file
- GitHub releases notes
- Project documentation

Big thanks to everyone helping make Aether Keeper better! 🙏

---

## License

By contributing, you agree that your contributions will be licensed under Apache-2.0.

