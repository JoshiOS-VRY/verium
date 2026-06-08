import { describe, expect, it } from "vitest";
import {
  coinHasStoredWallet,
  fullNodeWalletExists,
  isCoinSetupComplete,
  isCoinWalletReady,
  resolveEffectiveWalletMode,
} from "@/lib/setup";

describe("isCoinWalletReady", () => {
  const emptyPrefs = { setup_completed: false, setup_completed_by_coin: {} };

  it("marks full-node ready when wallet.dat exists", () => {
    expect(
      isCoinWalletReady("vericoin", emptyPrefs, {
        walletMode: "full_node",
        hasLightWallet: false,
        hasFullNodeWallet: true,
      }),
    ).toBe(true);
  });

  it("marks full-node not ready without wallet.dat when light wallet exists", () => {
    expect(
      isCoinWalletReady("vericoin", emptyPrefs, {
        walletMode: "full_node",
        hasLightWallet: true,
        hasFullNodeWallet: false,
      }),
    ).toBe(false);
  });

  it("marks light mode ready only with a stored light wallet", () => {
    expect(
      isCoinWalletReady("vericoin", emptyPrefs, {
        walletMode: "light",
        hasLightWallet: true,
        hasFullNodeWallet: true,
      }),
    ).toBe(true);
    expect(
      isCoinWalletReady("vericoin", emptyPrefs, {
        walletMode: "light",
        hasLightWallet: false,
        hasFullNodeWallet: true,
      }),
    ).toBe(false);
  });

  it("does not treat setup_completed as ready without keys for the active mode", () => {
    const completedPrefs = {
      setup_completed: true,
      setup_completed_by_coin: { verium: true as const },
    };
    expect(
      isCoinWalletReady("verium", completedPrefs, {
        walletMode: "light",
        hasLightWallet: false,
        hasFullNodeWallet: true,
      }),
    ).toBe(false);
    expect(
      isCoinWalletReady("verium", completedPrefs, {
        walletMode: "full_node",
        hasLightWallet: false,
        hasFullNodeWallet: true,
      }),
    ).toBe(true);
  });
});

describe("resolveEffectiveWalletMode", () => {
  it("prefers persisted light mode over hub full-node selection", () => {
    expect(resolveEffectiveWalletMode("light", "full_node")).toBe("light");
  });

  it("uses hub light selection before prefs catch up", () => {
    expect(resolveEffectiveWalletMode("full_node", "light")).toBe("light");
  });
});

describe("isCoinSetupComplete", () => {
  const emptyPrefs = { setup_completed: false, setup_completed_by_coin: {} };

  it("treats an on-disk light wallet as setup complete", () => {
    expect(
      isCoinSetupComplete("verium", emptyPrefs, { hasLightWallet: true }),
    ).toBe(true);
  });
});

describe("coinHasStoredWallet", () => {
  it("is true when either wallet type exists", () => {
    expect(coinHasStoredWallet({ hasLightWallet: true })).toBe(true);
    expect(coinHasStoredWallet({ hasFullNodeWallet: true })).toBe(true);
    expect(coinHasStoredWallet({})).toBe(false);
  });
});

describe("fullNodeWalletExists", () => {
  it("detects datadir and legacy Qt wallets", () => {
    expect(fullNodeWalletExists({ exists: true, legacy_wallet_detected: false })).toBe(
      true,
    );
    expect(fullNodeWalletExists({ exists: false, legacy_wallet_detected: true })).toBe(
      true,
    );
    expect(fullNodeWalletExists({ exists: false, legacy_wallet_detected: false })).toBe(
      false,
    );
  });
});
