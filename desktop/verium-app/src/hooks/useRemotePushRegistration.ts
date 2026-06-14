import { useEffect, useRef } from 'react';
import { useQuery } from '@tanstack/react-query';
import { coinQueryKey } from '@/lib/coin/profile';
import { useCoinWalletMode } from '@/hooks/useWalletMode';
import { useUserPreferences } from '@/lib/user-preferences';
import {
  pushHeartbeatDevice,
  pushRegistrationConfigured,
  pushSyncDevice,
  pushUnregisterDevice,
  pushWatchScripthashCounts,
} from '@/lib/push-notifications/register';
import { isWalletUnlocked } from '@/lib/wallet-unlock';
import { rpcGetWalletInfo } from '@/lib/rpc/client';

const HEARTBEAT_MS = 15 * 60 * 1000;
const TOKEN_POLL_MS = 30 * 60 * 1000;
const TOKEN_RETRY_MS = 2_000;
const TOKEN_RETRY_ATTEMPTS = 5;
const WALLET_POLL_MS = 10_000;

async function obtainPushToken(): Promise<string | null> {
  try {
    const { requestPermission, getToken } = await import('tauri-plugin-mobile-push-api');
    const { granted } = await requestPermission();
    if (!granted) return null;
    const token = await getToken();
    return token?.trim() ? token.trim() : null;
  } catch {
    return null;
  }
}

async function obtainPushTokenWithRetry(): Promise<string | null> {
  for (let i = 0; i < TOKEN_RETRY_ATTEMPTS; i++) {
    const token = await obtainPushToken();
    if (token) return token;
    if (i < TOKEN_RETRY_ATTEMPTS - 1) {
      await new Promise((r) => setTimeout(r, TOKEN_RETRY_MS));
    }
  }
  return null;
}

/**
 * iOS light wallet: register APNs device token + receive scripthashes with
 * push.vericonomy.com so incoming VRM/VRC alerts work when the app is killed.
 */
export function useRemotePushRegistration(): void {
  const veriumMode = useCoinWalletMode('verium');
  const vericoinMode = useCoinWalletMode('vericoin');
  const prefs = useUserPreferences((s) => s.prefs);

  const notifyVrm = prefs.notify_on_vrm_received !== false;
  const notifyVrc = prefs.notify_on_vrc_received !== false;
  const vericoinEnabled = prefs.vericoin_enabled !== false;
  const wantsPush = veriumMode.mobileOnly && (notifyVrm || (notifyVrc && vericoinEnabled));

  const configured = useQuery({
    queryKey: ['push-registration-configured'],
    queryFn: pushRegistrationConfigured,
    staleTime: 60_000,
    enabled: wantsPush,
  });

  const vrmWallet = useQuery({
    queryKey: coinQueryKey('verium', 'getwalletinfo'),
    queryFn: () => rpcGetWalletInfo('verium'),
    enabled: wantsPush && veriumMode.isLight,
    staleTime: 5_000,
    refetchInterval: wantsPush ? WALLET_POLL_MS : false,
  });

  const vrcWallet = useQuery({
    queryKey: coinQueryKey('vericoin', 'getwalletinfo'),
    queryFn: () => rpcGetWalletInfo('vericoin'),
    enabled: wantsPush && vericoinMode.isLight && vericoinEnabled,
    staleTime: 5_000,
    refetchInterval: wantsPush ? WALLET_POLL_MS : false,
  });

  const vrmUnlocked = isWalletUnlocked(vrmWallet.data);
  const vrcUnlocked = isWalletUnlocked(vrcWallet.data);
  const anyUnlocked = vrmUnlocked || vrcUnlocked;

  // Re-register when balance/history/indexing changes so new funded addresses are watched.
  const walletSyncKey = [
    vrmWallet.data?.balance,
    vrmWallet.data?.txcount,
    vrmWallet.data?.light_syncing,
    vrcWallet.data?.balance,
    vrcWallet.data?.txcount,
    vrcWallet.data?.light_syncing,
  ].join('|');

  const tokenRef = useRef<string | null>(null);
  const lastRegisteredRef = useRef(false);
  const lastSyncKeyRef = useRef<string>('');

  useEffect(() => {
    if (!wantsPush || configured.data !== true) return;

    let cancelled = false;

    const sync = async (heartbeat = false) => {
      if (cancelled) return;

      if (!anyUnlocked) {
        return;
      }

      let token = tokenRef.current;
      if (!token) {
        token = await obtainPushTokenWithRetry();
        if (cancelled || !token) {
          console.warn('remote push: no APNs token (check notification permission)');
          return;
        }
        tokenRef.current = token;
      }

      try {
        if (heartbeat) {
          await pushHeartbeatDevice(token);
        } else {
          await pushSyncDevice(token);
          const [vrm, vrc] = await pushWatchScripthashCounts();
          console.info('remote push registered', { vrmScripthashes: vrm, vrcScripthashes: vrc });
          if (notifyVrm && vrm === 0) {
            console.warn('remote push: no VRM addresses registered — unlock and wait for sync');
          }
        }
        lastRegisteredRef.current = true;
        lastSyncKeyRef.current = walletSyncKey;
      } catch (e) {
        console.warn('push registration failed', e);
      }
    };

    const syncKeyChanged = walletSyncKey !== lastSyncKeyRef.current;
    if (!lastRegisteredRef.current || syncKeyChanged) {
      void sync(false);
    }

    const heartbeatId = window.setInterval(() => void sync(true), HEARTBEAT_MS);
    const tokenPollId = window.setInterval(async () => {
      const fresh = await obtainPushToken();
      if (fresh && fresh !== tokenRef.current) {
        tokenRef.current = fresh;
        await sync(false);
      }
    }, TOKEN_POLL_MS);

    return () => {
      cancelled = true;
      window.clearInterval(heartbeatId);
      window.clearInterval(tokenPollId);
    };
  }, [
    wantsPush,
    configured.data,
    anyUnlocked,
    walletSyncKey,
    notifyVrm,
    notifyVrc,
    vericoinEnabled,
    vrmUnlocked,
    vrcUnlocked,
  ]);

  // Drop server registration when user disables remote push entirely.
  useEffect(() => {
    if (wantsPush || configured.data !== true) return;

    const token = tokenRef.current;
    if (!token || !lastRegisteredRef.current) return;

    void pushUnregisterDevice(token)
      .catch(() => {})
      .finally(() => {
        lastRegisteredRef.current = false;
      });
  }, [wantsPush, configured.data]);
}
