# Chain Forge - Testing & Verification Guide

This guide provides step-by-step instructions to test and verify Chain Forge functionality.

## Prerequisites Installation

This section provides complete installation instructions for all required tools.

### 1. Install Rust (Required)

**macOS/Linux:**
```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts, then reload your shell
source $HOME/.cargo/env

# Verify installation
rustc --version   # Should show 1.75 or later
cargo --version
```

**Windows:**
- Download and run [rustup-init.exe](https://rustup.rs/)
- Follow the installer prompts
- Restart your terminal
- Verify: `rustc --version`

**Troubleshooting:**
- If `rustc` not found, add to PATH: `export PATH="$HOME/.cargo/bin:$PATH"`
- Add to shell profile: `echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc`

### 2. Install Node.js (Required)

**Recommended: Use nvm (Node Version Manager)**

**macOS/Linux:**
```bash
# Install nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash

# Reload shell
source ~/.bashrc  # or ~/.zshrc

# Install Node.js 20 LTS
nvm install 20
nvm use 20

# Verify
node --version    # Should show v20.x.x
npm --version
```

**Windows:**
- Download [nvm-windows](https://github.com/coreybutler/nvm-windows/releases)
- Install and run: `nvm install 20 && nvm use 20`
- Or download directly from [nodejs.org](https://nodejs.org/)

**Alternative (Direct Install):**
- macOS: `brew install node@20`
- Ubuntu: `sudo apt install nodejs npm`
- Fedora: `sudo dnf install nodejs`

**Verify:**
```bash
node --version    # Should be v18.x.x or v20.x.x
```

### 3. Enable Yarn 4 (Required)

Node.js 16.10+ includes Corepack, which manages Yarn:

```bash
# Enable Corepack (one-time setup)
corepack enable

# Verify Yarn is available
yarn --version    # Should show 4.0.2

# If Yarn 4 not showing, prepare it
corepack prepare yarn@4.0.2 --activate
```

**If Corepack not available:**
```bash
npm install -g corepack
corepack enable
```

**Verify:**
```bash
yarn --version    # Must show 4.x.x
```

### 4. Install Solana CLI Tools (Required)

**macOS/Linux:**
```bash
# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Add to PATH (add to ~/.bashrc or ~/.zshrc for persistence)
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# Verify installation
solana --version                 # Should show 2.x.x or later
solana-test-validator --version  # Should show 2.x.x or later
```

**Persistent PATH (Important!):**
```bash
# For Bash
echo 'export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# For Zsh (macOS default)
echo 'export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

**Windows:**
```powershell
# Run in PowerShell as Administrator
cmd /c "curl https://release.solana.com/stable/solana-install-init-x86_64-pc-windows-msvc.exe --output C:\solana-install-tmp\solana-install-init.exe --create-dirs"
C:\solana-install-tmp\solana-install-init.exe stable
```

**Verify:**
```bash
solana --version
solana-test-validator --version
which solana-test-validator  # Should show path
```

**Troubleshooting:**
```bash
# If command not found
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# Check if binary exists
ls -la $HOME/.local/share/solana/install/active_release/bin/

# Reinstall if needed
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
```

### 5. Install Build Tools (Platform-specific)

**macOS:**
```bash
# Xcode Command Line Tools (if not already installed)
xcode-select --install
```

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev
```

**Fedora/RHEL:**
```bash
sudo dnf install -y gcc gcc-c++ make openssl-devel
```

**Windows:**
```bash
# Install Visual Studio Build Tools
# Download from: https://visualstudio.microsoft.com/downloads/
# Select "Desktop development with C++"
```

### Quick Verification

Run this to verify everything is installed:

```bash
echo "Checking prerequisites..."
echo ""

echo -n "Rust: "
rustc --version 2>/dev/null || echo "❌ NOT INSTALLED"

echo -n "Cargo: "
cargo --version 2>/dev/null || echo "❌ NOT INSTALLED"

echo -n "Node.js: "
node --version 2>/dev/null || echo "❌ NOT INSTALLED"

echo -n "Yarn: "
yarn --version 2>/dev/null || echo "❌ NOT INSTALLED"

echo -n "Solana: "
solana --version 2>/dev/null || echo "❌ NOT INSTALLED"

echo -n "Solana Test Validator: "
solana-test-validator --version 2>/dev/null || echo "❌ NOT INSTALLED"

echo ""
echo "If any tool shows '❌ NOT INSTALLED', follow the installation steps above."
```

### Minimum Version Requirements

| Tool | Minimum Version | Recommended |
|------|----------------|-------------|
| Rust | 1.75.0 | Latest stable |
| Node.js | 18.0.0 | 20.x LTS |
| Yarn | 4.0.0 | 4.0.2 |
| Solana CLI | 2.0.0 | Latest stable |

## Test Suite Overview

We'll test:
1. ✅ Rust workspace builds
2. ✅ Unit tests pass
3. ✅ CLI functionality
4. ✅ TypeScript package
5. ✅ Demo application

---

## 1. Build & Test Rust Workspace

### Step 1.1: Build All Crates

```bash
cd chain-forge
cargo build --workspace --release
```

**Expected Output:**
```
   Compiling chain-forge-common v0.1.0
   Compiling chain-forge-config v0.1.0
   ...
   Finished `release` profile [optimized] target(s) in 2m 15s
```

**✅ Success Criteria:** No compilation errors

### Step 1.2: Run Unit Tests

```bash
cargo test --workspace
```

**Expected Output:**
```
running 100+ tests
test chain_forge_common::error::tests::test_error_display ... ok
test chain_forge_common::types::tests::test_network_from_str ... ok
...
test result: ok. 100+ passed; 0 failed; 0 ignored
```

**✅ Success Criteria:** All tests pass

### Step 1.3: Check Code Quality

```bash
# Format check
cargo fmt --all -- --check

# Linting
cargo clippy --workspace --all-features -- -D warnings
```

**✅ Success Criteria:** No warnings or errors

---

## 2. Test CLI Functionality

### Step 2.1: Install CLI

```bash
cargo install --path chains/solana/crates/cli
```

**Expected Output:**
```
  Installing chain-forge-solana-cli v0.1.0
   Installed package `chain-forge-solana-cli v0.1.0` (executable `cf-solana`)
```

### Step 2.2: Verify Installation

```bash
cf-solana --version
which cf-solana
```

**Expected Output:**
```
chain-forge-solana-cli 0.1.0
/Users/yourname/.cargo/bin/cf-solana
```

### Step 2.3: Test Help Command

```bash
cf-solana --help
```

**Expected Output:**
```
Chain Forge - Solana local development tool

Usage: cf-solana <COMMAND>

Commands:
  start     Start local Solana test validator
  accounts  List all generated accounts
  fund      Fund an account with SOL
  config    Show current configuration
  stop      Stop the running validator
  help      Print this message
```

### Step 2.4: Start Validator (Terminal 1)

```bash
# Open Terminal 1
cf-solana start --accounts 5 --balance 50
```

**Expected Output:**
```
🔑 Mnemonic: test test test test test test test test test test test junk
   Save this mnemonic to recover your accounts!

🚀 Starting Solana test validator on port 8899...
⏳ Waiting for validator to be ready...
✅ Validator is ready!

💰 Setting 5 accounts to 50 SOL each...
✅ All accounts funded!

🎉 Solana test validator is running!
   RPC URL: http://localhost:8899

💡 Tip: Keep this terminal open to keep the validator running
   Run 'cf-solana accounts' in another terminal to see your accounts
```

**✅ Success Criteria:**
- Mnemonic displayed
- Validator starts without errors
- All 5 accounts funded
- Process stays running

### Step 2.5: List Accounts (Terminal 2)

```bash
# Open Terminal 2
cf-solana accounts
```

**Expected Output:**
```
┌───────┬────────────────────────────────────────────────┬────────────────┐
│ Index │ Public Key                                     │ Balance (SOL)  │
├───────┼────────────────────────────────────────────────┼────────────────┤
│ 0     │ 7xJ5DxPkW3r4b6X9Qn8YZmK1vL2wN5cT4aB3sM9hR    │ 50.00          │
│ 1     │ 8kL2EwQlX4s5c7Y0Ro9ZnN2xM3yO6dU5bC4tN0iS    │ 50.00          │
│ 2     │ 9mN3FyRmY5t6d8A1Sp0aOq3zN4xP7eV6cD5uO1jT    │ 50.00          │
│ 3     │ 1oP4GzSnZ6u7e9B2Tq1bPr4aO5yQ8fW7dE6vP2kU    │ 50.00          │
│ 4     │ 2pQ5HaTnA7v8f0C3Ur2cQs5bP6zR9gX8eF7wQ3lV    │ 50.00          │
└───────┴────────────────────────────────────────────────┴────────────────┘
```

**✅ Success Criteria:**
- 5 accounts displayed
- Each has 50 SOL balance
- All have unique addresses

### Step 2.6: Fund an Account (Terminal 2)

```bash
# Copy a public key from the accounts list
cf-solana fund 7xJ5DxPkW3r4b6X9Qn8YZmK1vL2wN5cT4aB3sM9hR 25
```

**Expected Output:**
```
💰 Requesting airdrop of 25 SOL to 7xJ5...
✅ Airdrop successful!
   Signature: 5KcT...
   New balance: 75 SOL
```

### Step 2.7: Verify Balance Update (Terminal 2)

```bash
cf-solana accounts
```

**Expected:** First account now shows 75 SOL (or close to it)

### Step 2.8: Stop Validator (Terminal 1)

```bash
# In Terminal 1, press Ctrl+C
```

**Expected Output:**
```
^C
🛑 Solana test validator stopped
```

---

## 3. Test TypeScript Package

### Step 3.1: Build TypeScript Package

```bash
cd npm/@chain-forge/solana
yarn install
yarn build
```

**Expected Output:**
```
> @chain-forge/solana@0.1.0 build
> tsc

✨  Done
```

**✅ Success Criteria:** `dist/` directory created with compiled files

### Step 3.2: Verify Build Output

```bash
ls -la dist/
```

**Expected:**
```
client.d.ts
client.js
index.d.ts
index.js
types.d.ts
types.js
```

---

## 4. Run Demo Application

### Step 4.1: Navigate to Simple Demo

```bash
cd ../../..  # Back to chain-forge root
cd examples/simple-demo
```

### Step 4.2: Install Dependencies

```bash
yarn install
```

**Expected Output:**
```
added X packages in Ys
```

### Step 4.3: Run Demo

```bash
yarn start
```

**Expected Output:**
```
🎯 Chain Forge - Simple Demo
============================

📦 Configuration:
   - Accounts: 3
   - Initial Balance: 100 SOL
   - Port: 8899

🚀 Starting validator...
✅ Validator started!

📋 Generated Accounts:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Account 0:
  Address: 7xJ5DxPkW3r4b6X9Qn8YZmK1vL2wN5cT4aB3sM9hR
  Balance: 100.00 SOL
  Path: m/44'/501'/0'/0'

Account 1:
  Address: 8kL2EwQlX4s5c7Y0Ro9ZnN2xM3yO6dU5bC4tN0iS
  Balance: 100.00 SOL
  Path: m/44'/501'/1'/0'

Account 2:
  Address: 9mN3FyRmY5t6d8A1Sp0aOq3zN4xP7eV6cD5uO1jT
  Balance: 100.00 SOL
  Path: m/44'/501'/2'/0'

🔧 Testing setBalance...
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Setting account 0 to 200 SOL...
✅ Result: Added 100 SOL (100 → 200 SOL)

Current balance: 200.00 SOL

Setting same account to 200 SOL again (should be idempotent)...
✅ Result: Balance already at 200 SOL (target: 200 SOL)

📊 Blockchain Info:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ Block Height: 42
✅ Slot: 45
✅ Version: 2.0.15

🎉 All tests passed!

Press Ctrl+C to stop the validator and exit...
```

**✅ Success Criteria:**
- Validator starts successfully
- 3 accounts generated with 100 SOL each
- setBalance works correctly
- Idempotent behavior demonstrated
- Blockchain info retrieved

### Step 4.4: Stop Demo

```bash
# Press Ctrl+C
```

**Expected Output:**
```
^C
🛑 Stopping validator...
✅ Validator stopped. Goodbye!
```

---

## 5. Manual Verification Steps

### Test 5.1: Deterministic Account Generation

```bash
# Terminal 1: Start with specific mnemonic
cf-solana start --mnemonic "test test test test test test test test test test test junk" --accounts 3

# Terminal 2: List accounts
cf-solana accounts --format json > accounts1.json

# Terminal 1: Stop (Ctrl+C)

# Terminal 1: Restart with same mnemonic
cf-solana start --mnemonic "test test test test test test test test test test test junk" --accounts 3

# Terminal 2: List accounts again
cf-solana accounts --format json > accounts2.json

# Compare
diff accounts1.json accounts2.json
```

**✅ Success Criteria:** Files are identical (same public keys generated)

### Test 5.2: Account Persistence

```bash
# Start validator
cf-solana start --accounts 2

# In another terminal, check storage
cat ~/.chain-forge/solana/accounts.json
```

**✅ Success Criteria:** File exists and contains 2 accounts in JSON format

### Test 5.3: Balance Idempotency

**Using CLI:**
```bash
# Terminal 1: Start validator
cf-solana start --accounts 1

# Terminal 2: Get initial balance
cf-solana accounts

# Fund to 100 SOL
cf-solana fund <ADDRESS> 100

# Fund to 100 SOL again (should add 100 more)
cf-solana fund <ADDRESS> 100

# Check balance (should be ~200 SOL)
cf-solana accounts
```

**Using TypeScript (in demo app):**
```typescript
// setBalance is idempotent
await client.setBalance(address, 100);  // Sets to 100
await client.setBalance(address, 100);  // Already 100, no change
await client.setBalance(address, 100);  // Still 100, no change
```

---

## 6. Integration Testing

### Test 6.1: Solana Web3.js Integration

Create a test file:

```javascript
// test-integration.js
const { Connection, PublicKey, LAMPORTS_PER_SOL } = require('@solana/web3.js');

async function test() {
  const connection = new Connection('http://localhost:8899', 'confirmed');

  // Get version
  const version = await connection.getVersion();
  console.log('Solana version:', version);

  // Get a test account from cf-solana
  const testPubkey = new PublicKey('7xJ5DxPkW3r4b6X9Qn8YZmK1vL2wN5cT4aB3sM9hR');

  // Get balance
  const balance = await connection.getBalance(testPubkey);
  console.log('Balance:', balance / LAMPORTS_PER_SOL, 'SOL');

  // Get block height
  const blockHeight = await connection.getBlockHeight();
  console.log('Block height:', blockHeight);
}

test().catch(console.error);
```

Run:
```bash
# Terminal 1: Start validator
cf-solana start

# Terminal 2: Run test
node test-integration.js
```

**✅ Success Criteria:** No errors, balance and block height displayed

---

## 7. Error Handling Tests

### Test 7.1: Validator Not Running

```bash
# Make sure no validator is running
cf-solana accounts
```

**Expected:** Message about validator not running or no accounts found

### Test 7.2: Invalid Address

```bash
# Start validator
cf-solana start

# In another terminal
cf-solana fund invalid-address 10
```

**Expected:** Error message about invalid public key

### Test 7.3: Port Already in Use

```bash
# Terminal 1: Start validator on port 8899
cf-solana start --port 8899

# Terminal 2: Try to start another on same port
cf-solana start --port 8899
```

**Expected:** Error about port already in use

---

## 8. Performance Tests

### Test 8.1: Startup Time

```bash
time cf-solana start --accounts 10
```

**Expected:** Complete in < 10 seconds

### Test 8.2: Account Generation Speed

Test with different account counts:

```bash
time cf-solana start --accounts 5
# Ctrl+C

time cf-solana start --accounts 10
# Ctrl+C

time cf-solana start --accounts 20
# Ctrl+C
```

**✅ Success Criteria:** Scales linearly with account count

---

## Troubleshooting

### Issue: `solana-test-validator: command not found`

**Solution:**
```bash
# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Add to PATH
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# Add to shell profile
echo 'export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Issue: Port 8899 already in use

**Solution:**
```bash
# Find process using port
lsof -i :8899

# Kill it
kill -9 <PID>

# Or use different port
cf-solana start --port 8900
```

### Issue: `cargo: command not found`

**Solution:**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Issue: TypeScript build fails

**Solution:**
```bash
cd npm/@chain-forge/solana
rm -rf node_modules package-lock.json
yarn install
yarn build
```

---

## Success Checklist

Use this checklist to verify everything works:

- [ ] Rust workspace builds successfully
- [ ] All 100+ unit tests pass
- [ ] CLI installs and runs
- [ ] Validator starts and stops cleanly
- [ ] Accounts are generated with correct balances
- [ ] Account funding works
- [ ] TypeScript package builds
- [ ] Demo application runs end-to-end
- [ ] Deterministic key generation verified
- [ ] setBalance idempotency confirmed
- [ ] Integration with @solana/web3.js works
- [ ] Error handling behaves correctly

---

## Next Steps

Once all tests pass:

1. **Initialize Git Repository**
   ```bash
   cd chain-forge
   git init
   git add .
   git commit -m "Initial commit: Chain Forge v0.1.0"
   ```

2. **Push to GitHub**
   ```bash
   git remote add origin https://github.com/yourusername/chain-forge
   git push -u origin main
   ```

3. **Verify CI/CD**
   - Check GitHub Actions run successfully
   - Review test results
   - Verify builds for all platforms

4. **Start Development**
   - Create feature branches
   - Make changes
   - Run tests
   - Submit PRs

---

## Test Report Template

Use this template to document test results:

```
# Chain Forge Test Report

**Date:** YYYY-MM-DD
**Version:** 0.1.0
**Tester:** Your Name
**Platform:** macOS/Linux/Windows

## Test Results

### Rust Tests
- Build: ✅ Pass / ❌ Fail
- Unit Tests: ✅ 100+ passed / ❌ X failed
- Clippy: ✅ No warnings / ❌ X warnings

### CLI Tests
- Installation: ✅ Pass / ❌ Fail
- Start Command: ✅ Pass / ❌ Fail
- Accounts Command: ✅ Pass / ❌ Fail
- Fund Command: ✅ Pass / ❌ Fail

### TypeScript Tests
- Build: ✅ Pass / ❌ Fail
- Demo App: ✅ Pass / ❌ Fail

### Issues Found
1. [Issue description]
2. [Issue description]

### Notes
[Any additional observations]
```

---

**Last Updated**: January 2026
**Version**: 0.1.0
