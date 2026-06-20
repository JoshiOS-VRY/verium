# Threat model — Vericonomy desktop wallet

STRIDE analysis for `desktop/verium-app` (Tauri 2 + React WebView + bundled nodes).

> **Qt / QML shell (`desktop/verium-qt`):** The native Qt port replaces the
> React-in-WebView frontend with QML rendered directly by the Qt engine. There
> is **no embedded browser engine and no hand-written C++**; the UI talks to the
> shared Rust core (`vericonomy-sdk`) through a cxx-qt bridge and the
> `vericonomy-desktop-host` layer. The WebView-related rows below (CSP, XSS →
> arbitrary invoke, `localStorage` history) are therefore **N/A** for that shell:
> there is no JS execution context, no DOM, and no `invoke()` reachable from web
> content. Q_INVOKABLE methods are only callable from the app's own compiled QML.
> All other rows (security policy, full-node path, secret store, updates) apply
> unchanged because that logic is shared.

## Components

| Component       | Description                                       |
| --------------- | ------------------------------------------------- |
| WebView UI      | React frontend in Tauri WebView                   |
| Tauri IPC       | ~185 invoke commands (Rust)                       |
| Security policy | `security_policy.rs` — 2FA + spending enforcement |
| Full-node path  | `veriumd` / `vericoind` JSON-RPC on loopback      |
| Light path      | Local signing + Electrum TLS                      |
| Secret store    | OS keychain + AES-GCM blobs                       |
| Updates         | CDN manifest + optional Tauri updater             |

## STRIDE summary

### Spoofing

| Threat                      | Mitigation                                                   |
| --------------------------- | ------------------------------------------------------------ |
| Phishing / clipboard hijack | Spending controls, address allowlist, re-check at send       |
| Fake Electrum server        | TLS to defaults; custom servers documented as higher risk    |
| Malicious update feed       | Updater signature verification (when enabled); hash manifest |

### Tampering

| Threat                  | Mitigation                                                           |
| ----------------------- | -------------------------------------------------------------------- |
| UI-only security bypass | Backend `security_policy` on send, export, restore, conf write       |
| RPC console abuse       | `rpc_raw_call` removed in production; method blocklist in dev builds |
| Audit log tampering     | Ed25519-signed append-only log                                       |

### Repudiation

| Threat                | Mitigation                                        |
| --------------------- | ------------------------------------------------- |
| Deny sensitive action | Signed audit log entries for export, send, backup |

### Information disclosure

| Threat                     | Mitigation                                                      |
| -------------------------- | --------------------------------------------------------------- |
| Plaintext secrets on disk  | Encrypted `secret_store` only; legacy migration removes mirrors |
| RPC password in UI         | Conf read returns redacted `rpcpassword` / `rpcuser`            |
| Logs / RPC console history | Sensitive RPC methods excluded from localStorage                |

### Denial of service

| Threat               | Mitigation                                           |
| -------------------- | ---------------------------------------------------- |
| Electrum rate limits | Server failover and backoff in `electrum/manager.rs` |
| Large chain sync     | Bootstrap import option                              |

### Elevation of privilege

| Threat                   | Mitigation                                                        |
| ------------------------ | ----------------------------------------------------------------- |
| XSS → arbitrary invoke   | (Tauri) CSP restricts `connect-src`; backend policy on sensitive commands. **N/A on the Qt/QML shell — no WebView, no JS, no DOM-reachable `invoke()`.** |
| Malware with disk access | Wallet passphrase is root of trust; documented limitation         |

## Out of scope (documented)

- Light-mode Electrum trust model (convenience tier)
- Manual hardware wallet PSBT workflow
- Attacker with physical access + keylogger
