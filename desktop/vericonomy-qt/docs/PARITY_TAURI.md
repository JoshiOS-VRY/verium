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

### P0 — Wallet access & migration (in progress)
- [x] Align `app_config_base` with Tauri
- [x] SDK light session (xprv + phrase import, unlock, sync, balance)
- [x] Setup hub: Import phrase/xprv + Unlock
- [x] `wallet_unlock`, `wallet_create_encrypted`, `wallet_restore` (host)
- [x] DashboardHero layout (wallet / market / mining-staking / network footer)
- [x] wallet.dat file picker in Setup UI
- [x] Wallet mode defaults to full node; light mode in Settings
- [ ] Light send + transaction history from Electrum cache
- [ ] `recovery_apply_hd_seed` (full-node phrase restore)

### P1 — Full-node parity (scaffold started)
- [x] `daemon_config.rs` + `commands/daemon.rs` stub
- [ ] DaemonManager spawn/stop/restart
- [ ] Pool miner sidecar
- [ ] Bootstrap import

### P2 — Security (scaffold started)
- [x] `commands/security.rs` stub
- [ ] 2FA, auto-lock, spending controls, scheduled backups

### P3 — Explorer & activity (partial)
- [x] Dashboard recent blocks feed
- [x] `commands/addressbook.rs` CRUD
- [x] `commands/pool.rs` stub
- [ ] Explorer tx/block/address detail routes
- [ ] Address book QML page

### P4 — Polish
- [ ] Inter + Lucide assets
- [ ] Installers + signing (scripts exist)

## Command surface (~200 Tauri commands)

See agent inventory in chat; grouped modules: `commands.rs`, `wallet_commands.rs`, `security_commands.rs`, `onboarding_commands.rs`, `network_mode_commands.rs`, `pool_miner.rs`, `mining_opt.rs`, `dace_commands.rs`.
