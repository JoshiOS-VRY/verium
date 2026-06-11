import { createContext, useCallback, useContext, useMemo, type ReactNode } from 'react';
import { ALL_COINS, getCoinProfile, type CoinId, type CoinProfile } from '@/lib/coin/profile';
import { useUserPreferences } from '@/lib/user-preferences';

interface CoinContextValue {
  activeCoin: CoinId;
  profile: CoinProfile;
  setActiveCoin: (coin: CoinId) => void;
  enabledCoins: CoinId[];
  isCoinEnabled: (coin: CoinId) => boolean;
}

const CoinContext = createContext<CoinContextValue | null>(null);

export function CoinProvider({ children }: { children: ReactNode }) {
  // Subscribe only to the three prefs fields this provider uses, not the whole
  // prefs object, so unrelated pref changes (theme, mining settings, …) don't
  // re-render the provider.
  const activeCoinPref = useUserPreferences((s) => s.prefs.active_coin);
  const veriumEnabled = useUserPreferences((s) => s.prefs.verium_enabled);
  const vericoinEnabled = useUserPreferences((s) => s.prefs.vericoin_enabled);
  const update = useUserPreferences((s) => s.update);

  const activeCoin: CoinId = activeCoinPref === 'vericoin' ? 'vericoin' : 'verium';

  const enabledCoins = useMemo(
    () =>
      ALL_COINS.filter((coin) => {
        if (coin === 'verium') return veriumEnabled !== false;
        return vericoinEnabled !== false;
      }),
    [veriumEnabled, vericoinEnabled]
  );

  const setActiveCoin = useCallback(
    (coin: CoinId) => {
      void update({ active_coin: coin });
    },
    [update]
  );

  const value = useMemo(
    () => ({
      activeCoin,
      profile: getCoinProfile(activeCoin),
      setActiveCoin,
      enabledCoins,
      isCoinEnabled: (coin: CoinId) => enabledCoins.includes(coin),
    }),
    [activeCoin, enabledCoins, setActiveCoin]
  );

  return <CoinContext.Provider value={value}>{children}</CoinContext.Provider>;
}

export function useActiveCoin(): CoinId {
  return useCoinContext().activeCoin;
}

export function useCoinProfile(): CoinProfile {
  return useCoinContext().profile;
}

export function useSetActiveCoin(): (coin: CoinId) => void {
  return useCoinContext().setActiveCoin;
}

export function useEnabledCoins(): CoinId[] {
  return useCoinContext().enabledCoins;
}

export function useCoinContext(): CoinContextValue {
  const ctx = useContext(CoinContext);
  if (!ctx) {
    throw new Error('useCoinContext must be used within CoinProvider');
  }
  return ctx;
}

export function useCoinProfileFor(coin: CoinId): CoinProfile {
  return getCoinProfile(coin);
}
