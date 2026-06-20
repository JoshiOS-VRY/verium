# iOS TestFlight release guide

> **Native Swift path:** The preferred iOS wallet is now `ios/VericonomyWallet/` (SwiftUI + UniFFI only). The Tauri iOS shell below is **deprecated** after side-by-side QA — see [`ios/VericonomyWallet/docs/ios-native-cutover.md`](../../../../ios/VericonomyWallet/docs/ios-native-cutover.md).

Vericonomy Wallet for iOS is a **light wallet only** (Electrum sync, no bundled
`veriumd` / `vericoind`). Bundle ID: `com.vericonomy.wallet.ios`. Marketing
version: **1.0.0**.

## Prerequisites

1. **Apple Developer Program** membership.
2. **App Store Connect** app record for `com.vericonomy.wallet.ios`.
3. **Xcode 15+** with iOS SDK (run builds via `scripts/with-xcode.sh`).
4. **Rust** with `aarch64-apple-ios` target (`rustup target add aarch64-apple-ios`).
5. Apple Developer **team ID** (10 characters, e.g. `396ZMFA3PP`). Optional if already set in
   `vericonomy-wallet.xcodeproj`. **Do not** use the literal placeholder `YOUR_TEAM_ID`.

```bash
export APPLE_DEVELOPMENT_TEAM=396ZMFA3PP   # your real team ID from Xcode → Accounts
```

6. Regenerate icons (flattens alpha onto `#000000` for App Store validation):

```bash
npm run icons:ios
```

7. If `src-tauri/gen/apple/` is missing, initialize once:

```bash
cd desktop/verium-app
npm install
npm run tauri:ios:init -- --ci
```

## Build an App Store IPA

Create `desktop/verium-app/.env` or `.env.local` **before** archiving (values are
embedded into the iOS binary at compile time):

```bash
VERICONOMY_PUSH_API_URL=https://push.vericonomy.com
VERICONOMY_PUSH_API_SECRET=<same as PUSH_API_SECRET on vericonomy-push droplet>
```

Remote lock-screen push is skipped if `VERICONOMY_PUSH_API_SECRET` is empty. See
[push-notifications.md](push-notifications.md).

```bash
cd desktop/verium-app
# Optional if DEVELOPMENT_TEAM is in the Xcode project:
export APPLE_DEVELOPMENT_TEAM=396ZMFA3PP
npm run ios:archive
```

If **archive already succeeded** but export failed (wrong team ID), fix the team and re-export only:

```bash
export APPLE_DEVELOPMENT_TEAM=396ZMFA3PP
npm run ios:export
```

This runs:

1. Frontend build (`npm run build:app`)
2. Rust compile for `aarch64-apple-ios`
3. `xcodebuild archive` → `src-tauri/gen/apple/build/vericonomy-wallet_iOS.xcarchive`
4. `xcodebuild -exportArchive` → `src-tauri/gen/apple/build/app-store/Vericonomy Wallet.ipa`

Before uploading, verify the **exported IPA** (not the raw xcarchive — export re-signs for App Store):

```bash
bash scripts/verify-ios-push-entitlements.sh
```

Must print `aps-environment: production` on the **IPA**. The xcarchive alone may still show
`development` until export; that is normal with automatic signing.

For ad-hoc device install (no TestFlight), use `npm run ios:ship` instead.

## Upload to TestFlight

1. Open **Transporter** (or Xcode **Organizer** → Archives → Distribute App).
2. Upload `Vericonomy Wallet.ipa`.
3. In **App Store Connect** → your app → **TestFlight**, wait for processing.
4. Complete **Export Compliance** (app uses standard TLS only;
   `ITSAppUsesNonExemptEncryption` is `false` in Info.plist).
5. Fill **App Privacy** (wallet data stored on device; no tracking).
6. Add **internal testers** and install from TestFlight.

Increment `CFBundleVersion` in
[`project.yml`](src-tauri/gen/apple/project.yml) (`CFBundleVersion: "2"`, etc.)
before each new upload while keeping `CFBundleShortVersionString` at `1.0.0`
until a marketing version bump.

## Regenerating Xcode metadata

Templates live in:

- [`src-tauri/gen/apple/project.yml`](src-tauri/gen/apple/project.yml)
- [`src-tauri/Info.ios.plist`](src-tauri/Info.ios.plist) (Tauri merge)
- [`src-tauri/gen/apple/vericonomy-wallet_iOS/Info.plist`](src-tauri/gen/apple/vericonomy-wallet_iOS/Info.plist)
- [`src-tauri/gen/apple/PrivacyInfo.xcprivacy`](src-tauri/gen/apple/PrivacyInfo.xcprivacy)

If you use XcodeGen: `xcodegen generate` from `src-tauri/gen/apple/`.

## Smoke test checklist (device)

Run on a **release** build before inviting external testers.

| Area       | Steps                                                                              |
| ---------- | ---------------------------------------------------------------------------------- |
| Onboarding | Create VRM wallet; import VRC phrase; funds disclaimer visible on setup hub        |
| Unlock     | Passphrase unlock; enable Face ID; disable Face ID                                 |
| Sync       | Balance appears via Electrum (vrm3 / vrc3 defaults)                                |
| Receive    | Show address, copy, QR                                                             |
| Send       | Small test send with default fee; confirm tx in history / explorer                 |
| QR scan    | Camera permission prompt; scan payment QR on send                                  |
| Explorer   | Tap tx / block / address links (in-app routes)                                     |
| Push       | Allow notifications; unlock wallet; kill app; receive test VRM → lock screen alert |
| Settings   | **No** “Check for updates” / download UI; Electrum server card works               |
| Deep links | Open `verium://…` payment URI → lands on send flow                                 |

### Send path (technical)

Light-wallet sends use parent-tx validation before signing:

- `prepare_utxos_for_signing` — fetches parent tx, normalizes txid/script
- `verify_signed_p2pkh_inputs` — local signature check before Electrum broadcast

If send fails, note the **in-app error** (pre-broadcast) vs Electrum error.

## Troubleshooting

| Issue                  | Action                                                                       |
| ---------------------- | ---------------------------------------------------------------------------- |
| Signing fails          | Confirm team ID, automatic signing, valid distribution cert in Keychain      |
| Biometrics unavailable | Confirm `NSFaceIDUsageDescription` in Info.plist; Face ID enrolled on device |
| Archive missing plist  | Run `npm run tauri:ios:init` or sync from `project.yml`                      |
| Stale UI on device     | `FORCE_RUST_REBUILD=1 npm run ios:archive`                                   |
| Stale icon in Transporter | Icons updated in `Assets.xcassets` but Xcode reused cached `Assets.car`. Run `IOS_FORCE_ICON_REBUILD=1 npm run ios:archive`, then `npm run ios:verify-icon` before uploading. Do **not** use `ios:export` alone after icon changes. |
| Invalid large app icon | Run `npm run icons:ios`, rebuild IPA (`npm run ios:archive`), re-upload      |
| iPad orientation error | `Info.plist` needs all four iPad orientations for multitasking; rebuild IPA  |
