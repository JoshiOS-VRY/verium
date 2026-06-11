# Wallet modes — full node vs light

## Exchange-grade default: full node

Full-node mode runs bundled `veriumd` / `vericoind` locally. The wallet:

- Stores keys in encrypted `wallet.dat` on disk
- Validates chain state from the P2P network
- Signs transactions locally via JSON-RPC
- Does not trust third-party index servers for UTXO data

**Centralized exchanges and institutional custody workflows should use full-node mode.**

## Convenience tier: light wallet

Light mode keeps keys encrypted on the device but uses Vericonomy Electrum servers for:

- Balance and transaction history
- UTXO discovery (gap scan)
- Transaction broadcast

Signing still happens locally; servers never receive mnemonics or private keys.

### Trust assumptions (light mode)

| Data      | Source         | Risk if server is malicious                         |
| --------- | -------------- | --------------------------------------------------- |
| UTXO set  | Electrum index | Incorrect balance, withheld UTXOs, fee manipulation |
| History   | Electrum index | Incomplete or misleading history                    |
| Chain tip | Electrum       | Stale tip could delay confirmation visibility       |

See [electrum-servers.md](./electrum-servers.md) for server list and TLS behavior.

## Setup default

The setup wizard defaults to **full node**. Light mode is under **Advanced: light wallet (convenience)** with an explicit warning.

## Switching modes

Use **Settings → Wallet mode**. Export your recovery phrase before switching from full node to light, or import your phrase into the node after switching from light to full.
