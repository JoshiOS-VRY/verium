import { useQuery } from '@tanstack/react-query';
import { useActiveCoin } from '@/lib/coin/context';
import { coinQueryKey } from '@/lib/coin/profile';
import { useChainSwitchTransition } from '@/hooks/useChainSwitchTransition';
import { useWalletMode } from '@/hooks/useWalletMode';
import { rpcGetBlockchainInfo, rpcGetWalletInfo } from '@/lib/rpc/client';

/**
 * True briefly after the active coin changes until core wallet/chain queries
 * have settled for the new coin. Used for sidebar + main-content switch UX.
 *
 * Intentionally does not wait on explorer blocks (slow); cached coordinator
 * data is enough for the shell to feel responsive.
 */
export function useAppCoinSwitchTransition(): boolean {
  const coin = useActiveCoin();
  const { isLight } = useWalletMode();

  const blockchain = useQuery({
    queryKey: coinQueryKey(coin, 'getblockchaininfo'),
    queryFn: () => rpcGetBlockchainInfo(coin),
    enabled: !isLight,
    refetchInterval: false,
    staleTime: 30_000,
  });

  const wallet = useQuery({
    queryKey: coinQueryKey(coin, 'getwalletinfo'),
    queryFn: () => rpcGetWalletInfo(coin),
    refetchInterval: false,
    staleTime: 10_000,
  });

  const chainReady = isLight || (!blockchain.isLoading && !blockchain.isFetching);
  const walletReady = !wallet.isLoading && !wallet.isFetching;

  return useChainSwitchTransition(coin, {
    isReady: chainReady && walletReady,
  });
}
