# Remote push notifications (iOS)

Lock-screen alerts when the app is killed require the Vericonomy push service
(`vericonomy-push`) plus APNs. The wallet registers your device token and
receive-address scripthashes while unlocked.

## Server

Deploy `vericonomy-push` on your droplet (see that repo’s README). Production
API: `https://push.vericonomy.com`.

## iOS app build

Set in `.env` or `.env.local` before `tauri ios build` (embedded at compile
time for the Rust HTTP client):

```bash
VERICONOMY_PUSH_API_URL=https://push.vericonomy.com
VERICONOMY_PUSH_API_SECRET=<same as PUSH_API_SECRET on server>
```

If `VERICONOMY_PUSH_API_SECRET` is unset, remote push registration is skipped
(in-app polling notifications still work while the app is open).

## Apple Developer / `.p8` key (server only)

The `.p8` file is configured on the **push service**, not in this wallet repo.
Full steps: `vericonomy-push/docs/apns-p8-setup.md`

Quick summary:

1. [Apple Developer → Keys](https://developer.apple.com/account/resources/authkeys/list) → create APNs key → download `AuthKey_XXXXX.p8` (once)
2. Copy to droplet: `/opt/vericonomy-push/secrets/apns_key.p8`
3. Set in push service `.env`: `APNS_KEY_ID`, `APNS_TEAM_ID`, `APNS_KEY_P8_HOST_PATH`
4. `docker compose --profile workers up -d --build`
5. TestFlight: `APNS_USE_SANDBOX=false` + entitlements `aps-environment` = `production`

## Per-chain toggles

Settings → Notifications:

- **Notify when VRM is received** → `notify_vrm` on the server
- **Notify when VRC is received** → `notify_vrc` on the server (only if VeriCoin is enabled)

Registration runs when the wallet is unlocked; unregister on lock.

## Privacy

The push service stores your APNs device token and scripthashes (derived from
public receive addresses). Keys never leave the device.
