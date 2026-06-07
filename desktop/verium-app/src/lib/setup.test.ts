import { describe, expect, it } from "vitest";
import { fullNodeWalletExists, isCoinWalletReady } from "@/lib/setup";

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
