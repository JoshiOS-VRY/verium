# CEX submission package — index

Package for exchange listing diligence reviews.

## Contents

1. [Security model](../SECURITY.md) — promises and trade-offs
2. [Threat model](./THREAT_MODEL.md) — STRIDE per component
3. [Data classification](./DATA_CLASSIFICATION.md) — persistence map
4. [Key lifecycle](./KEY_LIFECYCLE.md) — generate → seal → unlock → wipe
5. [Wallet modes](../../desktop/verium-app/docs/wallet-modes.md) — full node recommended
6. [Release verification](../RELEASE_SECURITY.md) — hashes, cosign, signing
7. [Incident response](./INCIDENT_RESPONSE.md) — disclosure policy
8. [Security test plan](./SECURITY_TEST_PLAN.md) — automated + manual cases
9. SBOM — `sbom-npm.json` + `sbom-cargo.json` (tagged releases)
10. Third-party audit report — attach when available

## Recommended exchange configuration

- **Wallet mode:** Full node
- **2FA:** Enabled for all gated actions
- **Spending controls:** Daily caps + allowlist for hot wallet operations
- **Backups:** Scheduled `wallet.dat` + independent recovery phrase custody

## Known limitations (disclose)

- Light wallet trusts Electrum index servers (convenience tier)
- Hardware wallets: manual xpub / PSBT workflow
- Verium BIP44 coin type not registered with SLIP-44
- Alpha builds may lack platform code signing until production promotion

## Audit engagement

Engage a third-party firm on:

- Tauri IPC boundary
- Full-node send / export / backup paths
- Release supply chain

Light mode: document as out-of-scope or limited-scope per [wallet-modes.md](../../desktop/verium-app/docs/wallet-modes.md).
