# Data classification

## Critical (key material)

| Data | Location | Protection |
| --- | --- | --- |
| BIP39 mnemonic (light) | `secure/light-wallet-keystore.enc` | Argon2id + AES-GCM; RAM while unlocked |
| `wallet.dat` keys | `<datadir>/wallet.dat` | AES-256 via node passphrase |
| TOTP secret | `secure/two-factor-config.enc` | AES-GCM via OS keychain master |
| PIN hash | `secure/passkey-config.enc` | Argon2id hash |
| WIF / xprv (transient) | Process memory | `zeroize` where applicable |

## Sensitive (credentials & config)

| Data | Location | Protection |
| --- | --- | --- |
| RPC password | `<datadir>/vericonomy.conf` | Not returned to UI (redacted on read) |
| Wallet passphrase (forever unlock) | OS keychain per datadir | OS-protected |
| 2FA recovery code hashes | Encrypted blob | SHA-256 hashes only |
| Encrypted cloud backup | User-selected path | Separate backup password |

## Internal (operational)

| Data | Location | Protection |
| --- | --- | --- |
| UTXO cache (light) | `light-cache-*.sqlite` | Watch-only chain data |
| Audit log | `secure/audit-log.enc` | Ed25519-signed entries |
| Address book | `secure/address-book.enc` | AES-GCM |
| User preferences | `secure/user-preferences.enc` | AES-GCM |

## Public / low sensitivity

| Data | Location | Notes |
| --- | --- | --- |
| Chain blocks | `<datadir>/blocks/` | Public chain data |
| RPC console history | `localStorage` | Sensitive methods filtered |
| Explorer / pool API responses | Transient HTTP | No secrets |

## Legacy (migrated on load)

| File | Action |
| --- | --- |
| `two_factor.json` | Migrated → deleted |
| `passkey.json` | Migrated → deleted |
| `light-keystore.json` | Migrated → deleted |
