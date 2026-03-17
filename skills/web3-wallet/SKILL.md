---
name: web3-wallet
version: 1.0.0
description: "web3 wallet: Create, import, show, export, and reset wallets."
metadata:
  openclaw:
    category: "web3"
    requires:
      bins: ["web3"]
      skills: ["web3-shared"]
---

# web3 wallet

> **PREREQUISITE:** Read `../web3-shared/SKILL.md` for key management, global flags, and security rules.

Manage wallet lifecycle — create keys, import existing keys, view addresses, export mnemonics, and reset the keystore.

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `create` | Generate a new wallet with random EVM keys |
| `import-key <CHAIN>` | Import a private key for a specific chain (reads from stdin) |
| `import-mnemonic` | Import a BIP-39 mnemonic phrase (reads from stdin) — coming Phase 2 |
| `show` | Show wallet information (addresses, chains, source) |
| `addresses` | Show all derived addresses |
| `export` | Export mnemonic to stdout (requires `--confirm`) |
| `reset` | Delete the encrypted keystore |

## Usage

### Create a new wallet

```bash
web3 wallet create
web3 wallet create --force    # overwrite existing wallet
```

### Import a private key

```bash
echo "0xYOUR_PRIVATE_KEY" | web3 wallet import-key ethereum
echo "0xYOUR_PRIVATE_KEY" | web3 wallet import-key ethereum --force
```

### Show wallet info

```bash
web3 wallet show
web3 wallet show --output json
```

### Show all addresses

```bash
web3 wallet addresses
web3 wallet addresses --output json
```

### Export mnemonic

```bash
web3 wallet export --confirm
```

### Reset wallet

```bash
web3 wallet reset           # interactive confirmation prompt
web3 wallet reset --force   # skip confirmation
```

## Flags

| Subcommand | Flag | Required | Default | Description |
|------------|------|----------|---------|-------------|
| `create` | `--force` | — | `false` | Overwrite existing wallet |
| `import-key` | `<CHAIN>` | ✓ | — | Chain to import the key for (e.g., `ethereum`) |
| `import-key` | `--force` | — | `false` | Overwrite existing wallet |
| `import-mnemonic` | `--force` | — | `false` | Overwrite existing wallet |
| `export` | `--confirm` | ✓ | — | Confirm you want to display the mnemonic |
| `reset` | `--force` | — | `false` | Skip confirmation prompt |

## Tips

- Wallet keys are encrypted with AES-256-GCM and stored via the platform keyring (with encrypted file fallback).
- `WEB3_PRIVATE_KEY` env var overrides the stored wallet when set.
- Only EVM chains are supported for key import in Phase 1. Solana/Sui key import coming later.
- The same EVM key works across all EVM chains (Ethereum, Polygon, Arbitrum, Base).

> [!CAUTION]
> `export` displays your mnemonic in plain text — never run this in shared terminals or logs.
> `reset` permanently deletes your encrypted keystore — all private keys will be lost. Confirm with the user before executing either command.

## See Also

- [web3-shared](../web3-shared/SKILL.md) — Global flags, security rules, and config
- [web3-balance](../web3-balance/SKILL.md) — Check balances after creating a wallet
- [web3-send](../web3-send/SKILL.md) — Send tokens from your wallet
