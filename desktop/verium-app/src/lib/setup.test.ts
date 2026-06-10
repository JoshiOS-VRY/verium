import { describe, expect, it } from "vitest";
import {
  coinHasStoredWallet,
  fullNodeWalletExists,
  getHubCardState,
  isCoinSetupComplete,
  isCoinWalletReady,
  isProfileOpenable,
  resolveCoinOpenMode,
  resolveEffectiveWalletMode,
} from "@/lib/setup";
import { profileWalletPresence } from "@/hooks/useSetupHubProfiles";
import type { WalletProfile } from "@/lib/wallet-profile";

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

describe("resolveCoinOpenMode", () => {
  it("does not cross-fallback to full_node when hub is light without light keys", () => {
    expect(
      resolveCoinOpenMode("light", {
        hasLightWallet: false,
        hasFullNodeWallet: true,
      }),
    ).toBeNull();
  });

  it("cross-fallback opens full-node when allowed (coin switcher)", () => {
    expect(
      resolveCoinOpenMode(
        "light",
        { hasLightWallet: false, hasFullNodeWallet: true },
        { allowCrossModeFallback: true },
      ),
    ).toBe("full_node");
  });

  it("respects hub full-node choice when wallet.dat exists", () => {
    expect(
      resolveCoinOpenMode("full_node", {
        hasLightWallet: false,
        hasFullNodeWallet: true,
      }),
    ).toBe("full_node");
  });

  it("opens light wallet when hub is light and keystore exists", () => {
    expect(
      resolveCoinOpenMode("light", {
        hasLightWallet: true,
        hasFullNodeWallet: true,
      }),
    ).toBe("light");
  });

  it("returns null when no wallet keys exist", () => {
    expect(
      resolveCoinOpenMode("full_node", {
        hasLightWallet: false,
        hasFullNodeWallet: false,
      }),
    ).toBeNull();
  });
});

describe("isProfileOpenable", () => {
  it("is true when mode mismatches but full-node keys exist", () => {
    expect(
      isProfileOpenable({
        ...mockProfile({ light: false, full_node: true }),
        mode: "light",
        coin: "vericoin",
      }),
    ).toBe(true);
  });

  it("is false when light keys exist on disk but keystore is unreadable", () => {
    expect(
      isProfileOpenable({
        ...mockProfile({ light: true, full_node: false, openable: false }),
        mode: "light",
        light_keystore_health: "unreadable",
      }),
    ).toBe(false);
  });

  it("is true when backend reports openable with decryptable light keys only", () => {
    expect(
      isProfileOpenable({
        ...mockProfile({ light: true, full_node: false }),
        mode: "full_node",
        openable: true,
        light_keystore_health: "ok",
        coin: "verium",
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

function mockProfile(
  partial: Partial<WalletProfile["keys_present"]> & {
    legacy?: boolean;
    openable?: boolean;
    light_keystore_health?: WalletProfile["light_keystore_health"];
  },
): WalletProfile {
  const light = partial.light ?? false;
  const fullNode = partial.full_node ?? false;
  const legacy = partial.legacy ?? false;
  return {
    coin: "verium",
    mode: "light",
    keys_present: {
      full_node: fullNode,
      light,
    },
    legacy: {
      detected: legacy,
      path: null,
    },
    hd: { has_mnemonic_backup: false },
    onboarding: { phase: "not_started" },
    intent: "fresh_install",
    ready: false,
    openable:
      partial.openable ??
      (light === true || fullNode === true || legacy === true),
    light_keystore_health:
      partial.light_keystore_health ?? (light ? "ok" : "missing"),
  };
}

describe("getHubCardState", () => {
  it("shows cross-mode label when full_node hub selected but light wallet on disk", () => {
    const state = getHubCardState({
      checking: false,
      walletMode: "full_node",
      presence: { hasLightWallet: true, hasFullNodeWallet: false },
      profile: mockProfile({ light: true, full_node: false, openable: true }),
    });
    expect(state.statusLabel).toBe("Light wallet on device");
    expect(state.actionLabel).toBe("Open");
  });

  it("shows Open when profile is openable regardless of hub mode", () => {
    const state = getHubCardState({
      checking: false,
      walletMode: "full_node",
      presence: { hasLightWallet: true, hasFullNodeWallet: false },
      profile: {
        ...mockProfile({ light: true, full_node: false }),
        mode: "full_node",
        openable: true,
        ready: false,
        light_keystore_health: "ok",
      },
    });
    expect(state.actionLabel).toBe("Open");
  });

  it("shows Import phrase when keystore is unreadable", () => {
    const state = getHubCardState({
      checking: false,
      walletMode: "light",
      presence: { hasLightWallet: true, hasFullNodeWallet: false },
      profile: {
        ...mockProfile({ light: true, full_node: false }),
        mode: "light",
        ready: false,
        openable: false,
        light_keystore_health: "unreadable",
      },
    });
    expect(state.statusLabel).toBe("Recovery required");
    expect(state.actionLabel).toBe("Import phrase");
    expect(state.ready).toBe(false);
  });

  it("full-node hub ignores unreadable light artifacts when wallet.dat exists", () => {
    const state = getHubCardState({
      checking: false,
      walletMode: "full_node",
      presence: { hasLightWallet: true, hasFullNodeWallet: true },
      profile: {
        ...mockProfile({ light: true, full_node: true }),
        mode: "light",
        ready: false,
        openable: true,
        light_keystore_health: "unreadable",
      },
    });
    expect(state.statusLabel).toBe("Ready");
    expect(state.actionLabel).toBe("Open");
  });
});

describe("profileWalletPresence", () => {
  it("derives light and full-node presence from wallet profile", () => {
    expect(
      profileWalletPresence(
        mockProfile({ light: true, full_node: true, legacy: false }),
      ),
    ).toEqual({
      hasLightWallet: true,
      hasFullNodeWallet: true,
    });
    expect(
      profileWalletPresence(
        mockProfile({ light: false, full_node: false, legacy: true }),
      ),
    ).toEqual({
      hasLightWallet: false,
      hasFullNodeWallet: true,
    });
    expect(profileWalletPresence(undefined)).toEqual({
      hasLightWallet: false,
      hasFullNodeWallet: false,
    });
  });
});

describe("hub readiness from wallet profile", () => {
  const emptyPrefs = { setup_completed: false, setup_completed_by_coin: {} };

  it("marks light hub card ready when profile reports light keys", () => {
    const { hasLightWallet, hasFullNodeWallet } = profileWalletPresence(
      mockProfile({ light: true, full_node: true }),
    );
    expect(
      isCoinWalletReady("verium", emptyPrefs, {
        walletMode: "light",
        hasLightWallet,
        hasFullNodeWallet,
      }),
    ).toBe(true);
  });

  it("shows not ready in light mode when only full-node keys exist", () => {
    const { hasLightWallet, hasFullNodeWallet } = profileWalletPresence(
      mockProfile({ light: false, full_node: true }),
    );
    expect(
      isCoinWalletReady("verium", emptyPrefs, {
        walletMode: "light",
        hasLightWallet,
        hasFullNodeWallet,
      }),
    ).toBe(false);
  });
});
