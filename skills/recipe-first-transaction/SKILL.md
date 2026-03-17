---
name: recipe-first-transaction
version: 1.0.0
description: "Getting started: Create a wallet and send your first transaction."
metadata:
  openclaw:
    category: "recipe"
    domain: "web3"
    requires:
      bins: ["web3"]
      skills: ["web3-shared", "web3-wallet", "web3-send", "web3-balance"]
---

# First Transaction

> **PREREQUISITE:** Load the following skills to execute this recipe: `web3-shared`, `web3-wallet`, `web3-send`, `web3-balance`

Walk through wallet creation to your first on-chain transaction.

## Steps

1. **Create a wallet:**

   ```bash
   web3 wallet create
   ```

2. **Show your address** (share this to receive funds):

   ```bash
   web3 wallet show
   ```

3. **Check your balance** (you need funds before sending):

   ```bash
   web3 balance --chain ethereum
   ```

4. **Dry-run a send** (simulate without spending):

   ```bash
   web3 send 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD18 0.01 --dry-run --chain ethereum
   ```

5. **Confirm with the user**, then execute the real send:

   ```bash
   web3 send 0x742d35Cc6634C0532925a3b844Bc9e7595f2bD18 0.01 --chain ethereum
   ```

6. **Check transaction status:**

   ```bash
   web3 tx status <HASH> --chain ethereum
   ```

## Tips

- Use `--network testnet` for experimentation without real funds.
- If the balance is zero, the user needs to fund the wallet address first (from an exchange or another wallet).
- Always dry-run before sending real value.
