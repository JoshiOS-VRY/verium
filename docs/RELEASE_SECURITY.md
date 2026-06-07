# Vericonomy Wallet — release verification

## Official channels

- GitHub Releases: `desktop-v*` tags on the [verium repository](https://github.com/JoshiOS-VRY/verium/releases)
- Update manifest: `https://files.vericonomy.com/vrm/VERSION_VRM.json`

## Verify downloads

### 1. SHA-256 hashes

Each release includes `release-hashes.json` with SHA-256 digests per installer.

```powershell
# Windows PowerShell
Get-FileHash .\Vericonomy_Wallet_*.exe -Algorithm SHA256
```

Compare the output to the value in `release-hashes.json`.

### 2. Sigstore cosign (tagged releases)

Tagged `desktop-v*` releases are signed with cosign. Verify a blob:

```bash
cosign verify-blob --certificate-oidc-issuer-regexp ".*" \
  --certificate-identity-regexp ".*" \
  --signature Vericonomy_Wallet_*.exe.sig \
  Vericonomy_Wallet_*.exe
```

### 3. Platform code signing (production)

Production releases should be:

| Platform | Expected signature |
| --- | --- |
| Windows | Authenticode (Publisher: Vericonomy) |
| macOS | Developer ID + notarization |
| Linux `.deb` | GPG package signature (when enabled) |

Alpha builds may be unsigned; verify hashes and cosign instead.

### 4. In-app installer verification

The wallet embeds `release-hashes.json` for bundled sidecar SHA-256 checks. Use **Settings → Security → Verify installation**.

## Auto-update

`tauri-plugin-updater` is integrated but disabled until a production minisign keypair is configured:

1. Generate keys: `npm run tauri signer generate -- -w ~/.tauri/vericonomy.key`
2. Set `plugins.updater.pubkey` in `tauri.conf.json`
3. Set `plugins.updater.active` to `true`
4. Configure CI secret `TAURI_SIGNING_PRIVATE_KEY` for release builds
5. Publish signed update JSON to the updater endpoint

## Maintainer CI secrets

| Secret | Purpose |
| --- | --- |
| `TAURI_SIGNING_PRIVATE_KEY` | Tauri updater + bundle signing |
| `APPLE_CERTIFICATE` / `APPLE_PASSWORD` | macOS notarization (when configured) |
| `WINDOWS_CERTIFICATE` | Authenticode (when configured) |
| Cosign OIDC (GitHub Actions) | Release blob attestation |

Never commit private keys or passphrases to the repository.
