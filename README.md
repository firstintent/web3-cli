# web3-cli

A multi-chain Web3 wallet CLI for AI agents. Unified interface to manage wallets, check balances, send transactions, and interact with smart contracts across multiple blockchains.

## Supported Chains

| Chain | Type | Chain ID |
|-------|------|----------|
| Ethereum | EVM | 1 |
| Polygon | EVM | 137 |
| Arbitrum | EVM | 42161 |
| Base | EVM | 8453 |
| Solana | — | — |
| Sui | — | — |

## Installation

**Requirements:** Rust 1.88.0+

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
# Binary at ./target/release/web3
```

## Usage

```bash
web3 [OPTIONS] <COMMAND>
```

### Global Options

| Flag | Description |
|------|-------------|
| `--output <FORMAT>` | Output format: `table` (default) or `json` |
| `--chain <CHAIN>` | Override default chain |
| `--network <NETWORK>` | Override default network |

### Wallet Management

```bash
web3 wallet create                     # Create a new wallet
web3 wallet import-key ethereum        # Import a private key
web3 wallet import-mnemonic            # Import a BIP-39 mnemonic
web3 wallet show                       # Display wallet info
web3 wallet addresses                  # List all addresses
web3 wallet export                     # Export keys (use with caution)
web3 wallet reset                      # Reset wallet
```

### Balance

```bash
web3 balance                           # Native token balance
web3 balance --token <CONTRACT>        # ERC-20 token balance
web3 balance all                       # All balances
web3 --chain polygon balance           # Balance on a specific chain
```

### Send Transactions

```bash
web3 send <TO> <AMOUNT>                # Send native token
web3 send <TO> <AMOUNT> --token <ADDR> # Send ERC-20 token
web3 send <TO> <AMOUNT> --dry-run      # Simulate without sending
```

### Sign Messages

```bash
web3 sign message <MESSAGE>            # Sign a message
```

### Transaction Info

```bash
web3 tx status <HASH>                  # Check transaction status
web3 tx history                        # View transaction history
```

### Smart Contract Interaction (EVM)

```bash
web3 evm call <CONTRACT> <METHOD> [ARGS...]   # Read (view/pure)
web3 evm send <CONTRACT> <METHOD> [ARGS...]   # Write (state-changing)
```

### Chain-Specific Commands

```bash
# Solana
web3 solana transfer <TO> <AMOUNT>
web3 solana invoke <PROGRAM> <DATA>

# Sui
web3 sui transfer <TO> <AMOUNT>
web3 sui call <PACKAGE> <MODULE> <FUNCTION> [ARGS...]
```

### Configuration

```bash
web3 config show                       # Show current configuration
web3 config set default_chain ethereum # Set default chain
web3 chain list                        # List supported chains
web3 chain info ethereum               # Chain details
web3 validate <ADDRESS>                # Validate address & detect chain
```

## Output Formats

**Table** (default) — human-readable tabular output.

**JSON** — structured envelope with metadata:

```json
{
  "ok": true,
  "chain": "ethereum",
  "network": "mainnet",
  "data": { ... }
}
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Transaction error |
| 2 | Auth / key error |
| 3 | Validation error |
| 4 | Network error |
| 5 | Internal error |

## Security

- Private keys encrypted with AES-256-GCM
- Platform-native secure storage via keyring (with encrypted file fallback)
- Sensitive data zeroed from memory after use
- Restrictive file permissions (0600) on key material

## Configuration

Config is stored at `~/.config/web3-cli/config.json`. Each chain supports multiple RPC endpoints for failover.

## License

MIT
