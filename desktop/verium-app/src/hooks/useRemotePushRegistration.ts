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
} from '@/lib/push-notifications/register';
import { isWalletUnlocked } from '@/lib/wallet-unlock';
import { rpcGetWalletInfo } from '@/lib/rpc/client';

const HEARTBEAT_MS = 15 * 60 * 1000;
const TOKEN_POLL_MS = 30 * 60 * 1000;

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
  const wantsPush =
    veriumMode.mobileOnly &&
    (notifyVrm || (notifyVrc && vericoinEnabled));

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
  });

  const vrcWallet = useQuery({
    queryKey: coinQueryKey('vericoin', 'getwalletinfo'),
    queryFn: () => rpcGetWalletInfo('vericoin'),
    enabled: wantsPush && vericoinMode.isLight && vericoinEnabled,
    staleTime: 5_000,
  });

  const vrmUnlocked = isWalletUnlocked(vrmWallet.data);
  const vrcUnlocked = isWalletUnlocked(vrcWallet.data);
  const anyUnlocked = vrmUnlocked || vrcUnlocked;

  const tokenRef = useRef<string | null>(null);
  const lastRegisteredRef = useRef(false);

  useEffect(() => {
    if (!wantsPush || configured.data !== true) return;

    let cancelled = false;

    const sync = async (heartbeat = false) => {
      if (cancelled) return;

      if (!anyUnlocked) {
        const token = tokenRef.current;
        if (token && lastRegisteredRef.current) {
          try {
            await pushUnregisterDevice(token);
          } catch {
            // best effort
          }
          lastRegisteredRef.current = false;
        }
        return;
      }

      let token = tokenRef.current;
      if (!token) {
        token = await obtainPushToken();
        if (cancelled || !token) return;
        tokenRef.current = token;
      }

      try {
        if (heartbeat) {
          await pushHeartbeatDevice(token);
        } else {
          await pushSyncDevice(token);
        }
        lastRegisteredRef.current = true;
      } catch (e) {
        console.warn('push registration failed', e);
      }
    };

    void sync(false);

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
    notifyVrm,
    notifyVrc,
    vericoinEnabled,
    vrmUnlocked,
    vrcUnlocked,
  ]);
}
