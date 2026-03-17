---
name: web3-balance
version: 1.0.0
description: "web3 balance: Check native token and ERC-20 token balances."
metadata:
  openclaw:
    category: "web3"
    requires:
      bins: ["web3"]
      skills: ["web3-shared"]
---

# web3 balance

> **PREREQUISITE:** Read `../web3-shared/SKILL.md` for key management, global flags, and security rules.

Check native token and ERC-20 token balances. This is a **read-only** command — safe to run without user confirmation.

## Subcommands

| Subcommand | Description |
|------------|-------------|
| _(none)_ | Check native token balance (ETH, MATIC, etc.) |
| `token` | Check a specific ERC-20 token balance |
| `all` | Check all balances (native + known tokens) |

## Usage

### Native balance (default)

```bash
web3 balance --chain ethereum
web3 balance --chain polygon --output json
web3 balance --chain ethereum --address 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD18
```

### Token balance

```bash
web3 balance token 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 --chain ethereum
web3 balance token 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 --chain ethereum --address 0x742d...
```

### All balances

```bash
web3 balance all --chain ethereum
```

## Flags

| Flag | Required | Default | Description |
|------|----------|---------|-------------|
| `--address <ADDR>` | — | wallet address | Address to check (defaults to your wallet) |
| `--chain <CHAIN>` | — | config default | Chain to query |
| `--network <NETWORK>` | — | config default | Network (`mainnet` or `testnet`) |
| `--output <FORMAT>` | — | `table` | Output format: `table` or `json` |

For the `token` subcommand:

| Arg | Required | Default | Description |
|-----|----------|---------|-------------|
| `<CONTRACT>` | ✓ | — | Token contract address or mint |
| `--address <ADDR>` | — | wallet address | Address to check |

## Tips

- Balance amounts are returned in human-readable units (e.g., `1.5 ETH`), not in wei/lamports.
- Use `--address` to check any address without needing a wallet.
- `all` currently shows only native balance (token discovery coming in a later phase).

## See Also

- [web3-shared](../web3-shared/SKILL.md) — Global flags, security rules, and config
- [web3-wallet](../web3-wallet/SKILL.md) — Create a wallet before checking balances
- [web3-send](../web3-send/SKILL.md) — Send tokens after checking your balance
