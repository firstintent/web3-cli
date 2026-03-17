---
name: web3-shared
version: 1.0.0
description: "web3 CLI: Shared patterns for key management, global flags, and security rules."
metadata:
  openclaw:
    category: "web3"
    requires:
      bins: ["web3"]
---

# web3 — Shared Reference

## Installation

The `web3` binary must be on `$PATH`. Install from source:

```bash
cd web3-cli && cargo install --path .
```

## Key Management

```bash
# Generate a new wallet (EVM keys, encrypted and stored securely)
web3 wallet create

# Or set a private key via environment variable (overrides stored wallet)
export WEB3_PRIVATE_KEY=0x...
```

Keys are stored using the platform keyring (macOS Keychain, Linux Secret Service) with an encrypted file fallback (AES-256-GCM). All key material is zeroed from memory after use.

## Global Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--output <FORMAT>` | `table` | Output format: `table` or `json` |
| `--chain <CHAIN>` | config default | Chain to use (overrides `default_chain` in config) |
| `--network <NETWORK>` | config default | Network to use: `mainnet` or `testnet` |

## CLI Syntax

```bash
web3 <command> [subcommand] [args] [flags]
```

## Supported Chains

| Chain | Aliases | Native Token | Status |
|-------|---------|-------------|--------|
| Ethereum | `ethereum`, `eth` | ETH | Stable |
| Polygon | `polygon`, `matic` | MATIC | Stable |
| Arbitrum | `arbitrum`, `arb` | ETH | Stable |
| Base | `base` | ETH | Stable |
| Solana | `solana`, `sol` | SOL | Preview |
| Sui | `sui` | SUI | Preview |

## JSON Output Envelope

All JSON output follows this envelope format:

```json
{
  "ok": true,
  "chain": "ethereum",
  "network": "mainnet",
  "data": { ... }
}
```

Error responses:

```json
{
  "ok": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "bad address",
    "category": "validation"
  }
}
```

## Security Rules

> [!CAUTION]
> web3 handles real private keys and real funds. Follow these rules strictly.

- **Always `--dry-run` first** before any value transfer (`send`, `evm send`)
- **Always confirm** with the user before executing send commands
- **Never output** private keys, mnemonics, or decryption keys in responses
- **Never log** sensitive key material
- **Validate addresses** before sending: `web3 validate <address>`
- **Check balance** before sending to avoid insufficient funds errors
- **Use testnet** for experimentation: `--network testnet`

## Exit Codes

| Code | Category | Description |
|------|----------|-------------|
| 0 | Success | Command completed successfully |
| 1 | Transaction | Transaction failed (insufficient funds, nonce, gas) |
| 2 | Auth/Key | Wallet not found, key decryption failed |
| 3 | Validation | Invalid address, unknown chain, bad input |
| 4 | Network | RPC connection failed, timeout |
| 5 | Internal | Unexpected error |

## Config Location

`~/.config/web3-cli/config.json`

```json
{
  "default_chain": "ethereum",
  "default_network": "mainnet",
  "chains": {
    "ethereum": {
      "rpc_urls": ["https://eth.drpc.org", "https://eth.llamarpc.com"],
      "explorer_url": "https://etherscan.io",
      "chain_id": 1
    }
  }
}
```

Each chain supports multiple RPC URLs from chainlist.org. Currently only the first URL is used (failover is planned).
