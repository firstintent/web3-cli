<h1 align="center">web3-cli</h1>

**One CLI for every chain — built for humans and AI agents.**<br>
Manage wallets, check balances, send transactions, and interact with smart contracts across Ethereum, Polygon, Arbitrum, Base, Solana, and Sui. Structured JSON output. Agent skills included.

```bash
cargo install --path .
```

## Contents

- [Quick Start](#quick-start)
- [Why web3-cli?](#why-web3-cli)
- [Supported Chains](#supported-chains)
- [Command Reference](#command-reference)
- [Output Formats](#output-formats)
- [AI Agent Skills](#ai-agent-skills)
- [Configuration](#configuration)
- [Security](#security)
- [Exit Codes](#exit-codes)
- [Development](#development)
- [License](#license)

## Quick Start

```bash
# 1. Create a wallet
web3 wallet create

# 2. Check your balance
web3 balance
web3 --chain polygon balance

# 3. Send a transaction (always dry-run first)
web3 send 0xRecipient 0.1 --dry-run
web3 send 0xRecipient 0.1
```

## Why web3-cli?

**For humans** — stop writing one-off scripts per chain. `web3` gives you a single CLI across six blockchains with `--help` on every command, `--dry-run` before moving funds, and table or JSON output.

**For AI agents** — every response is structured JSON with typed exit codes. Pair it with the included agent skills and your LLM can manage wallets and execute transactions without custom tooling.

> Give your AI agent the `web3-shared` skill and it can immediately manage wallets, check balances, and send transactions across chains — no custom tooling needed. Just paste the skill file into your agent's context and it has everything it needs: command syntax, security rules, output format, and error handling.

```bash
# Agent-friendly: structured JSON output with metadata
web3 --output json balance
# {"ok": true, "chain": "ethereum", "network": "mainnet", "data": {"balance": "1.5", ...}}

# Agent-friendly: deterministic exit codes for error handling
web3 send 0xBad 1.0; echo $?
# 3 — validation error (bad address)
```

## Supported Chains

| Chain | Type | Chain ID | Native Token | Status |
|-------|------|----------|-------------|--------|
| Ethereum | EVM | 1 | ETH | Stable |
| Polygon | EVM | 137 | MATIC | Stable |
| Arbitrum | EVM | 42161 | ETH | Stable |
| Base | EVM | 8453 | ETH | Stable |
| Solana | — | — | SOL | Preview |
| Sui | — | — | SUI | Preview |

## Command Reference

### Wallet Management

```bash
web3 wallet create                     # Create a new wallet
web3 wallet import-key ethereum        # Import a private key for a chain
web3 wallet import-mnemonic            # Import a BIP-39 mnemonic
web3 wallet show                       # Display wallet info
web3 wallet addresses                  # List all addresses across chains
web3 wallet export                     # Export keys (use with caution)
web3 wallet reset                      # Delete all stored keys
```

### Balance

```bash
web3 balance                           # Native token balance (default chain)
web3 balance --token 0xA0b8...eB48     # ERC-20 token balance
web3 balance all                       # All chain balances
web3 --chain polygon balance           # Balance on a specific chain
web3 --output json balance             # JSON output for scripting
```

### Send Transactions

```bash
web3 send 0xRecipient 0.5             # Send native token
web3 send 0xRecipient 100 --token 0xA0b8...eB48  # Send ERC-20 token
web3 send 0xRecipient 0.1 --dry-run   # Simulate without sending
web3 --chain polygon send 0xRecipient 10          # Send on a specific chain
```

### Sign Messages

```bash
web3 sign message "Hello, world"       # Sign a message with your wallet key
```

### Transaction Info

```bash
web3 tx status 0xabc123...            # Check transaction status
web3 tx history                        # View transaction history
```

### Smart Contract Interaction (EVM)

```bash
web3 evm call 0xContract balanceOf 0xAddress      # Read (view/pure)
web3 evm send 0xContract transfer 0xTo 1000       # Write (state-changing)
web3 evm send 0xContract approve 0xSpender 1000 --dry-run  # Simulate first
```

### Solana

```bash
web3 solana transfer <TO> <AMOUNT>
web3 solana invoke <PROGRAM> <DATA>
```

### Sui

```bash
web3 sui transfer <TO> <AMOUNT>
web3 sui call <PACKAGE> <MODULE> <FUNCTION> [ARGS...]
```

### Chain Info

```bash
web3 chain list                        # List all supported chains
web3 chain info ethereum               # Chain details (RPC, explorer, chain ID)
web3 validate 0xabc123...             # Validate address & detect chain
```

### Configuration Commands

```bash
web3 config show                       # Show current configuration
web3 config set default_chain polygon  # Set default chain
web3 config set default_network testnet  # Switch to testnet
```

### Global Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--output <FORMAT>` | `table` | Output format: `table` or `json` |
| `--chain <CHAIN>` | config default | Override default chain |
| `--network <NETWORK>` | config default | Override default network (`mainnet` or `testnet`) |

## Output Formats

**Table** (default) — human-readable tabular output:

```
Chain     Balance    Token
ethereum  1.5000     ETH
polygon   250.0000   MATIC
```

**JSON** (`--output json`) — structured envelope for scripting and agents:

```json
{
  "ok": true,
  "chain": "ethereum",
  "network": "mainnet",
  "data": {
    "balance": "1.5",
    "token": "ETH"
  }
}
```

Error responses follow the same envelope:

```json
{
  "ok": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "invalid ethereum address",
    "category": "validation"
  }
}
```

## AI Agent Skills

The repo ships 4 agent skills and 1 recipe — compact `SKILL.md` files that give an AI agent everything it needs to use web3-cli. See the full [Skills Index](docs/skills.md).

| Skill | Description |
|-------|-------------|
| [web3-shared](skills/web3-shared/SKILL.md) | Shared patterns: global flags, security rules, output format, exit codes |
| [web3-wallet](skills/web3-wallet/SKILL.md) | Wallet management: create, import, show, export, reset |
| [web3-balance](skills/web3-balance/SKILL.md) | Balance queries: native tokens and ERC-20 |
| [web3-send](skills/web3-send/SKILL.md) | Send transactions: native and ERC-20 with dry-run |
| [recipe-first-transaction](skills/recipe-first-transaction/SKILL.md) | Getting started: create a wallet and send your first transaction |

### How agents use skills

An AI agent can start using web3-cli by loading a skill file into its context. The `web3-shared` skill contains the command syntax, security rules, output format, and error codes — everything an agent needs to operate autonomously.

```bash
# Install skills into your agent framework
npx skills add https://github.com/anthropics/web3-cli

# Or copy individual skills
cp -r skills/web3-shared ~/.openclaw/skills/
cp -r skills/web3-send ~/.openclaw/skills/
```

The `web3-shared` skill includes an `install` block so compatible agent frameworks auto-install the CLI if `web3` isn't on PATH.

## Configuration

Config is stored at `~/.config/web3-cli/config.json`:

```json
{
  "default_chain": "ethereum",
  "default_network": "mainnet",
  "chains": {
    "ethereum": {
      "rpc_urls": ["https://eth.drpc.org", "https://eth.llamarpc.com"],
      "explorer_url": "https://etherscan.io",
      "chain_id": 1
    },
    "polygon": {
      "rpc_urls": ["https://polygon-rpc.com"],
      "explorer_url": "https://polygonscan.com",
      "chain_id": 137
    }
  }
}
```

Each chain supports multiple RPC URLs (sourced from [chainlist.org](https://chainlist.org)). Currently only the first URL is used — failover across URLs is planned.

### Environment Variables

| Variable | Description |
|----------|-------------|
| `WEB3_PRIVATE_KEY` | Private key override (bypasses stored wallet) |

## Security

web3-cli handles real private keys and real funds. Security is non-negotiable.

- **AES-256-GCM encryption** — all private keys encrypted at rest
- **Platform-native keyring** — macOS Keychain, Linux Secret Service, Windows Credential Manager; encrypted file fallback when keyring is unavailable
- **Memory zeroing** — all private keys, mnemonics, and decryption keys are zeroed after use via `zeroize`
- **File permissions** — restrictive 0600 permissions on all key material
- **Dry-run first** — `--dry-run` on every command that moves funds, so agents and humans can verify before committing
- **Address validation** — chain-specific format validation before any send
- **Input sanitization** — RPC URLs, file paths, and addresses validated against injection and path traversal

## Exit Codes

`web3` uses structured exit codes so scripts and agents can branch on the failure type without parsing stderr.

| Code | Meaning | Example cause |
|------|---------|---------------|
| `0` | Success | Command completed normally |
| `1` | Transaction error | Insufficient funds, nonce conflict, gas estimation failed |
| `2` | Auth / key error | Wallet not found, key decryption failed |
| `3` | Validation error | Invalid address, unknown chain, bad input |
| `4` | Network error | RPC connection failed, timeout |
| `5` | Internal error | Unexpected failure |

```bash
web3 send 0xRecipient 0.1 --dry-run
echo $?  # 0 — success

web3 send 0xBadAddress 1.0
echo $?  # 3 — validation error
```

## Development

**Requirements:** Rust 1.88.0+

```bash
cargo build --release          # Production build → target/release/web3
cargo clippy -- -D warnings    # Lint — zero warnings policy
cargo test                     # Unit + integration tests
cargo install --path .         # Install binary locally
```

Or build a debug binary:

```bash
cargo build
# Binary at ./target/debug/web3
```

## License

MIT
