---
name: web3-send
version: 1.0.0
description: "web3 send: Send native tokens or ERC-20 tokens to an address."
metadata:
  openclaw:
    category: "web3"
    requires:
      bins: ["web3"]
      skills: ["web3-shared"]
---

# web3 send

> **PREREQUISITE:** Read `../web3-shared/SKILL.md` for key management, global flags, and security rules.

Send native tokens (ETH, MATIC, etc.) or ERC-20 tokens to a recipient address.

> [!CAUTION]
> This is a **write** command that transfers real funds. **ALWAYS** run with `--dry-run` first, then confirm with the user before executing without `--dry-run`.

## Usage

```bash
web3 send <TO> <AMOUNT> [--token <CONTRACT>] [--dry-run] [--chain <CHAIN>]
```

## Flags

| Flag | Required | Default | Description |
|------|----------|---------|-------------|
| `<TO>` | ✓ | — | Recipient address |
| `<AMOUNT>` | ✓ | — | Amount to send (human-readable, e.g., `0.1` not wei) |
| `--token <CONTRACT>` | — | — | Token contract address (for ERC-20 transfers) |
| `--dry-run` | — | `false` | Simulate the transaction without sending |
| `--chain <CHAIN>` | — | config default | Chain to send on |
| `--network <NETWORK>` | — | config default | Network (`mainnet` or `testnet`) |
| `--output <FORMAT>` | — | `table` | Output format: `table` or `json` |

## Examples

### Send native token (dry-run first)

```bash
# Step 1: Always dry-run first
web3 send 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD18 0.1 --dry-run --chain ethereum

# Step 2: Confirm with user, then send for real
web3 send 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD18 0.1 --chain ethereum
```

### Send ERC-20 token

```bash
# Dry-run USDC transfer
web3 send 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD18 100 \
  --token 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 \
  --dry-run --chain ethereum

# Execute USDC transfer
web3 send 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD18 100 \
  --token 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 \
  --chain ethereum
```

### JSON output

```bash
web3 send 0x742d...5f2bD18 0.1 --dry-run --chain ethereum --output json
```

## Tips

- Amount is in human-readable units (e.g., `0.1` ETH), not in wei.
- Zero-value sends (`0` or `0.0`) trigger a warning — they still cost gas.
- Validate the recipient address first: `web3 validate <address>`
- Check your balance before sending: `web3 balance --chain <CHAIN>`
- The response includes a transaction hash and block explorer URL.

## Agent Workflow

When an agent needs to send funds, follow this sequence:

1. **Validate** the recipient: `web3 validate <TO>`
2. **Check balance**: `web3 balance --chain <CHAIN>`
3. **Dry-run**: `web3 send <TO> <AMOUNT> --dry-run --chain <CHAIN>`
4. **Confirm** with the user — show them the dry-run output
5. **Execute**: `web3 send <TO> <AMOUNT> --chain <CHAIN>`
6. **Verify**: `web3 tx status <HASH> --chain <CHAIN>`

## See Also

- [web3-shared](../web3-shared/SKILL.md) — Global flags, security rules, and config
- [web3-balance](../web3-balance/SKILL.md) — Check balance before sending
- [web3-wallet](../web3-wallet/SKILL.md) — Wallet must exist before sending
