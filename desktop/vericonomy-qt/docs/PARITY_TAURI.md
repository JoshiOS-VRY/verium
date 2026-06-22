# Qt wallet — Tauri parity roadmap

Goal: **absolute feature parity** with `desktop/verium-app` (Tauri + React). The Qt shell replaces Tauri; both share the same on-disk layout under `~/…/Vericonomy/desktop-app/` and the same `vericonomy-sdk` light-wallet stack.

## Migration from Tauri (balance preservation)

| Tauri mode | How balance is stored | Qt migration |
|------------|----------------------|--------------|
| **Light wallet** | `light-keystore.json` + Electrum UTXO cache | Unlock existing keystore **or** Security → Export xprv → Setup → Import xprv/phrase |
| **Full node** | `wallet.dat` in daemon datadir | Setup → Import `wallet.dat` (same path as Tauri) **or** use existing RPC if daemon already running |
| **Phrase (HD)** | `sethdseed` on full node | Setup → Restore 24-word phrase |
| **xprv only** | Light keystore or exported from full-node HD | Setup → paste xprv (Core HD paths) |

Storage paths (macOS): `~/Library/Application Support/Vericonomy/desktop-app/`

## Phase status

### P0 — Wallet access & migration
- [x] Align `app_config_base` with Tauri
- [x] SDK light session (xprv + phrase import, unlock, sync, balance)
- [x] Setup hub: Import phrase/xprv + Unlock
- [x] `wallet_unlock`, `wallet_create_encrypted`, `wallet_restore` (host)
- [x] DashboardHero layout (wallet / market / mining-staking / network footer)
- [x] wallet.dat file picker in Setup UI
- [x] Wallet mode defaults to full node; light mode in Settings
- [x] Light send + transaction history from Electrum/explorer cache
- [x] `recovery_apply_hd_seed` (full-node phrase restore)

### P1 — Full-node parity (scaffold started)
- [x] `daemon_config.rs` + `commands/daemon.rs`
- [x] DaemonManager spawn/stop/restart (`daemon_manager.rs`, binary detection)
- [x] Pool miner sidecar (`mining_supervisor.rs`, `commands/pool.rs`)
- [x] Bootstrap import (CDN download, local zip, extract/apply, restart)

### P2 — Security (partial)
- [x] 2FA enroll/verify/disable (TOTP, shared JSON store)
- [x] Auto-lock config + idle tracking
- [x] Spending controls (caps, allowlist, first-send flag)
- [x] Receive requests list (label/amount/message)
- [x] Light-wallet recovery export (phrase/xprv)
- [x] Full-node HD xprv export (`hd_wallet_export.rs`, Security → Export)
- [ ] Scheduled backups / mnemonic backup (full node)
- [ ] Audit log

### P3 — Explorer & activity (partial)
- [x] Dashboard recent blocks feed
- [x] `commands/addressbook.rs` CRUD
- [x] Address book QML page
- [x] Receive panel with QR + saved requests
- [ ] Explorer tx/block/address detail routes
- [ ] Binary Chain page

### P4 — Send UX (partial)
- [x] Fee rate field (light + full node path for light)
- [x] Payment URI QR codes + clipboard
- [x] Send confirm dialog + 2FA prompt at send time (single recipient)
- [ ] Multi-recipient single tx / coin control dialog
- [ ] QR scan for send address

### P5 — Polish
- [ ] Inter + Lucide assets (partial nav icons exist)
- [ ] Installers + signing (scripts exist)

## Command surface (~200 Tauri commands)

Grouped modules: `commands.rs`, `wallet_commands.rs`, `security_commands.rs`, `onboarding_commands.rs`, `network_mode_commands.rs`, `pool_miner.rs`, `mining_opt.rs`, `dace_commands.rs`.
