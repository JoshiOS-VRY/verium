import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Loader2, RefreshCw } from 'lucide-react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { MobileSettingsGroup } from '@/components/mobile/MobileSettingsGroup';
import { useCoinWalletMode } from '@/hooks/useWalletMode';
import { useEnabledCoins } from '@/lib/coin/context';
import { coinQueryKey, getCoinProfile, type CoinId } from '@/lib/coin/profile';
import { invalidateLightWalletQueries } from '@/lib/invalidate-wallet-queries';
import { lightWalletCopy } from '@/lib/light-wallet/copy';
import { lightWalletRescan } from '@/lib/light-wallet/client';
import { rpcGetWalletInfo } from '@/lib/rpc/client';

function useLightWalletUnlocked(coin: CoinId, enabled: boolean) {
  return useQuery({
    queryKey: coinQueryKey(coin, 'getwalletinfo'),
    queryFn: () => rpcGetWalletInfo(coin),
    enabled,
    staleTime: 10_000,
  });
}

export function LightWalletRescanCard({ mobileLayout = false }: { mobileLayout?: boolean }) {
  const enabledCoins = useEnabledCoins();
  const veriumMode = useCoinWalletMode('verium');
  const vericoinMode = useCoinWalletMode('vericoin');
  const queryClient = useQueryClient();
  const [successCoin, setSuccessCoin] = useState<CoinId | null>(null);

  const lightCoins: CoinId[] = enabledCoins.filter((c) => {
    if (c === 'verium') return veriumMode.isLight;
    if (c === 'vericoin') return vericoinMode.isLight;
    return false;
  });

  const vrmWallet = useLightWalletUnlocked('verium', lightCoins.includes('verium'));
  const vrcWallet = useLightWalletUnlocked('vericoin', lightCoins.includes('vericoin'));

  const rescan = useMutation({
    mutationFn: (coin: CoinId) => lightWalletRescan(coin),
    onSuccess: async (_, coin) => {
      setSuccessCoin(coin);
      await invalidateLightWalletQueries(queryClient, coin);
      await queryClient.invalidateQueries({
        queryKey: coinQueryKey(coin, 'wallet-cumulative-txs'),
      });
      await queryClient.invalidateQueries({
        predicate: (q) =>
          Array.isArray(q.queryKey) &&
          q.queryKey[0] === coin &&
          (q.queryKey[1] === 'listtransactions' || q.queryKey[1] === 'listunspent'),
      });
    },
    onError: () => setSuccessCoin(null),
  });

  if (veriumMode.isLoading || vericoinMode.isLoading || lightCoins.length === 0) {
    return null;
  }

  const walletForCoin = (coin: CoinId) =>
    coin === 'verium' ? vrmWallet.data : vrcWallet.data;

  const unlockedForCoin = (coin: CoinId) =>
    walletForCoin(coin)?.private_keys_enabled === true;

  const body = (
    <div className="flex flex-col gap-3">
      {lightCoins.map((coin) => {
        const profile = getCoinProfile(coin);
        const unlocked = unlockedForCoin(coin);
        const pending = rescan.isPending && rescan.variables === coin;

        return (
          <div
            key={coin}
            className={
              mobileLayout
                ? 'flex flex-col gap-2'
                : 'flex flex-wrap items-center justify-between gap-3 rounded-md border border-border bg-bg-subtle/40 px-3 py-2'
            }
          >
            <div className="min-w-0">
              <p className="text-sm font-medium text-fg">{profile.displayName}</p>
              {!unlocked && (
                <p className="text-xs text-fg-muted">{lightWalletCopy.rescanUnlockHint}</p>
              )}
            </div>
            <Button
              size={mobileLayout ? 'md' : 'sm'}
              variant="secondary"
              className={mobileLayout ? 'h-11 w-full rounded-xl' : ''}
              disabled={!unlocked || rescan.isPending}
              onClick={() => rescan.mutate(coin)}
            >
              {pending ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin" />
                  Rescanning…
                </>
              ) : (
                <>
                  <RefreshCw className="h-4 w-4" />
                  Rescan {profile.symbol}
                </>
              )}
            </Button>
          </div>
        );
      })}

      {successCoin && !rescan.isPending && (
        <p className="text-xs text-success">{lightWalletCopy.rescanSuccess}</p>
      )}
      {rescan.error && (
        <p className="text-xs text-danger">
          {rescan.error instanceof Error ? rescan.error.message : String(rescan.error)}
        </p>
      )}
    </div>
  );

  if (mobileLayout) {
    return (
      <MobileSettingsGroup
        title={lightWalletCopy.rescanTitle}
        description={lightWalletCopy.rescanDescription}
        defaultOpen={false}
      >
        {body}
      </MobileSettingsGroup>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>{lightWalletCopy.rescanTitle}</CardTitle>
        <CardDescription>{lightWalletCopy.rescanDescription}</CardDescription>
      </CardHeader>
      <CardContent>{body}</CardContent>
    </Card>
  );
}
