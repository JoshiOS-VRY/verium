/**
 * Light wallet poll cadence — tuned for high-capacity Vericonomy Electrum (vrm3/vrc3).
 */

/** Full balance sync (UTXO discovery + cache refresh). */
export const LIGHT_BALANCE_POLL_MS = 5_000;

/** Activity screen: pending history merge + tx list refresh. */
export const LIGHT_FOREGROUND_SYNC_MS = 3_000;

/** `getwalletinfo` steady poll (drives background UTXO/history sync). */
export const LIGHT_WALLET_INFO_POLL_MS = 5_000;

/** Shared wallet transaction list poll while the window is visible. */
export const LIGHT_TX_POLL_MS = 5_000;

/** Wallet tx poll when the window is hidden (notifications / chimes). */
export const LIGHT_TX_BACKGROUND_POLL_MS = 30_000;

/** Transactions page history query while Activity is open. */
export const LIGHT_HISTORY_POLL_MS = 5_000;

/** Electrum server status badge / tip-driven sync probe. */
export const LIGHT_SERVER_STATUS_POLL_MS = 10_000;

/** Incoming VRM/VRC watchers — Electrum pending merge + tx list read. */
export const LIGHT_INCOMING_NOTIFY_POLL_MS = 2_000;

/** Incoming notify poll while app is backgrounded (iOS may still suspend JS). */
export const LIGHT_INCOMING_NOTIFY_BACKGROUND_POLL_MS = 15_000;
