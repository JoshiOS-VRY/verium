# Key lifecycle

## Full-node wallet

1. **Generate** — `encryptwallet` RPC creates encrypted `wallet.dat`
2. **Unlock** — Passphrase via `walletpassphrase` (optional OS keychain cache)
3. **Sign** — Node holds decrypted keys in memory while unlocked
4. **Lock** — `walletlock` RPC or auto-lock idle timer
5. **Backup** — `backupwallet` RPC → `wallet.dat` copy
6. **Wipe** — Delete datadir; passphrase required to use backup

## Light wallet

1. **Generate / import** — BIP39 or xprv sealed with user passphrase (Argon2id + AES-GCM)
2. **Unlock** — Decrypted mnemonic in `SESSION_MNEMONICS` (configurable timeout, default 24h)
3. **Sign** — Local secp256k1 in `wallet/signer.rs`; Electrum receives raw tx only
4. **Lock** — `light_wallet_lock` clears session mnemonic
5. **Export** — Gated by 2FA + wallet passphrase (`recovery_export_seed`)

## HD recovery phrase (full-node upgrade)

1. Generated in setup wizard
2. Optional encrypted backup in `secure/hd-mnemonic-*.enc`
3. Applied via `sethdseed` RPC (gated by 2FA when restoring)

## Memory handling

- `Zeroizing<String>` for mnemonics in session map
- Passphrase buffers cleared after RPC calls where implemented
- No `mlock` / secure heap (documented residual risk)

## Residual risks

- Crash dumps may contain unlocked session secrets
- Attacker with live process memory can read unlocked mnemonic
- Forever-unlock keychain entries survive until cleared by user
