# Migrating your Tauri wallet to Qt

## Before you start

1. **Back up** your Tauri wallet: Settings → wallet backup, or copy `~/Library/Application Support/Vericonomy/desktop-app/`.
2. Note your **wallet mode** per coin: Settings shows Light vs Full node.
3. Qt reads the **same folder** as Tauri: `Vericonomy/desktop-app` (not `Verium/` alone).

## Light wallet (most common for quick setup)

### Option A — Reuse existing keystore (no re-import)

If you used Tauri light mode and have not changed passphrases:

1. Run Qt wallet.
2. Setup → **Unlock wallet** with your existing passphrase.
3. Qt loads `light-keystore.json` and syncs via Electrum (same as Tauri).

### Option B — Import xprv from Tauri Security export

1. In **Tauri**: Security → Export recovery → copy **HD master xprv** (or 24-word phrase).
2. In **Qt**: Setup → **Import wallet** → paste xprv or phrase + set passphrase.
3. Wait for sync to finish; balance matches after Electrum gap scan.

xprv uses **Core HD paths** (`m/0'/0'/n'`), not BIP44 — same as Tauri light import.

## Full-node wallet

### Option A — Same machine, daemon already synced

1. Ensure `vericonomy.conf` RPC credentials match your running `veriumd`.
2. Qt dashboard should show the same balance via `getwalletinfo` (no import needed).

### Option B — Import wallet.dat

1. Qt Setup → **Import wallet.dat** → select your backup file.
2. Qt stops the daemon, swaps `wallet.dat`, restarts, and runs `rescanblockchain`.
3. Unlock with the **passphrase from when that backup was made**.

### Option C — Full-node HD without phrase backup

1. Tauri Security → Export → **HD master xprv**.
2. Qt Setup → switch coin to **Light** → Import xprv.
3. Balance comes from Electrum instead of local chain scan.

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| Balance is 0 after import | Run manual rescan / wait for sync; check Electrum connectivity |
| "No wallet" on setup | Wrong mode — light needs import/unlock; full needs wallet.dat or running daemon |
| Qt does not see Tauri prefs | Confirm both use `~/Library/Application Support/Vericonomy/desktop-app/prefs.json` |
| Passphrase fails | Light vs full use different stores; use the passphrase for the store you imported |
