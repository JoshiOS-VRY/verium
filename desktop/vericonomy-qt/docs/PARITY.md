# Qt/QML port — feature checklist

Goal: **replace** the Tauri + React desktop shell with native Qt/QML. We are **porting** screens and behavior into Qt — not maintaining two apps in sync or delegating from Tauri.

## Done

- cxx-qt bridge (async Rust → QML)
- `vericonomy-desktop-host` (RPC, config, host commands)
- Design system + AppShell + navigation
- Layout fixes (`Card` sizing, list delegates)
- Live data: node, wallet, transactions, mining info, peers, logs, RPC console

## Port next (priority order)

1. **Send / receive** — address generation, QR, `sendtoaddress`
2. **Mining** — start/stop, thread control, hashrate (port `mining_supervisor` into host)
3. **Staking** — start/stop, stake weight (Vericoin)
4. **Setup wizard** — wallet create/restore, daemon bootstrap
5. **Settings** — prefs, daemon config, network mode
6. **Explorer** — indexer/explorer API clients
7. **Security** — promote shared logic to SDK; wire QML
8. **Address book, sign/verify, resources polish**
9. **Packaging** — installers, fonts (Inter), Lucide icons, signing

## Not required

- Tauri app continuing to build or delegate to host
- Feature-for-feature parity checks against the React UI
- Keeping both desktops shippable in parallel

## Run

```bash
cd vericonomy-wallet/desktop/verium-qt
export PATH="$(brew --prefix qt)/bin:$PATH"
cargo run
```
