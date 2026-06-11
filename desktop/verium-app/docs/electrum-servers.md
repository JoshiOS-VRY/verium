# Electrum servers (light wallet)

Light Wallet Mode connects to **existing** Electrum-compatible servers for Verium and VeriCoin.
The desktop wallet does not operate these servers.

## Default endpoints (mainnet)

| Chain    | Primary (TLS)                         | Failover (TLS)                        |
| -------- | ------------------------------------- | ------------------------------------- |
| Verium   | `electrumx-vrm1.vericonomy.com:51002` | `electrumx-vrm2.vericonomy.com:52002` |
| VeriCoin | `electrumx-vrc1.vericonomy.com:50012` | `electrumx-vrc2.vericonomy.com:50012` |

VRM WebSocket (not used by desktop wallet): `51004` / `52004` on the same hosts.

Override via **Settings → Light wallet → Custom servers**, or environment:

- `VERICONOMY_VRM_ELECTRUM_SERVERS` — comma-separated `host:port` or `tls://host:port`
- `VERICONOMY_VRC_ELECTRUM_SERVERS` — same format

## Protocol

- Electrum protocol **1.4** (newline-delimited JSON-RPC)
- Scripthash: `reverse_bytes(hex(SHA256(scriptPubKey)))`
- Required methods: see `src-tauri/src/chain/electrum/conformance.rs`

## Trust model

Servers provide UTXOs, history, headers, fee hints, and broadcast relay only.
Private keys and seeds never leave the device.

Light wallet mode is a **convenience tier** — exchanges and high-value custody should use
[full-node mode](./wallet-modes.md). Default servers use TLS (`rustls` + system roots).
Custom servers you configure are not pinned; use only hosts you trust.

Misbehaving or MITM-compromised servers could report false balances or withhold UTXOs.
Signing remains local, so servers cannot spend funds without your passphrase-unlocked keys.
