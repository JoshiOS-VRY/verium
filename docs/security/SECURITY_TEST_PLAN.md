# Security test plan

## Automated (CI)

| Test                | Command                                                              |
| ------------------- | -------------------------------------------------------------------- |
| Rust unit tests     | `cargo test --manifest-path desktop/verium-app/src-tauri/Cargo.toml` |
| Frontend unit tests | `npm test` in `desktop/verium-app`                                   |
| npm audit           | `npm audit --audit-level=high`                                       |
| cargo audit         | `cargo audit`                                                        |

## Backend policy (`security_policy.rs`)

| Case                             | Expected                            |
| -------------------------------- | ----------------------------------- |
| Send with 2FA enabled, no TOTP   | Rejected                            |
| Send with valid TOTP             | Allowed (if spending controls pass) |
| Daily cap exceeded               | Rejected                            |
| Allowlist mode, unknown address  | Rejected                            |
| `dumpprivkey` without TOTP       | Rejected                            |
| `restore_wallet` without TOTP    | Rejected                            |
| `write_verium_conf` without TOTP | Rejected                            |

## RPC surface

| Case                               | Expected                     |
| ---------------------------------- | ---------------------------- |
| `rpc_raw_call` in production build | Command not registered       |
| `dumpprivkey` via dev console      | Blocked by `rpc_guard`       |
| RPC history persistence            | Sensitive methods not stored |

## Cryptography

| Case                        | Expected                               |
| --------------------------- | -------------------------------------- |
| Invalid Base58Check address | Rejected at `validate_send_address`    |
| Signed tx output mismatch   | Rejected before broadcast (light mode) |
| Legacy plaintext 2FA file   | Migrated to encrypted store            |

## Manual / pentest checklist

- [ ] Invoke `send_to_address` via devtools without TOTP when 2FA on
- [ ] Modify frontend to skip SendPanel gates — confirm backend rejects
- [ ] Verify `read_verium_conf` redacts RPC secrets
- [ ] Confirm no `two_factor.json` after enrollment
- [ ] Installer hash verification on clean VM
- [ ] Update downgrade attempt (when updater enabled)

## Light mode (limited scope)

- [ ] Document Electrum MITM scenario
- [ ] Confirm keys not sent over Electrum protocol (packet capture)
