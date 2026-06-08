import { useEffect } from "react";
import { useQuery } from "@tanstack/react-query";

import { useLightServerConnected } from "@/hooks/useLightServerConnected";
import { useWalletMode } from "@/hooks/useWalletMode";
import { useWindowVisible } from "@/hooks/useWindowVisible";
import { coinQueryKey, type CoinId } from "@/lib/coin/profile";
import { pushChainTip, useChainTip } from "@/lib/chain-tip-store";
import { fetchExplorerBlocks } from "@/lib/explorer-api";
import { rpcGetBlockchainInfo } from "@/lib/rpc/client";

/**
 * Local tip for live block feeds: max(chain-tip watcher, `getblockchaininfo`).
 * The hero uses the same precedence; recent blocks must match or the table
 * lags while the watcher catches up to RPC.
 */
export function useEffectiveLocalChainTip(coin: CoinId) {
  const { isLight } = useWalletMode();
  const lightServer = useLightServerConnected();
  const visible = useWindowVisible();
  const chainTip = useChainTip(coin);

  const blockchain = useQuery({
    queryKey: coinQueryKey(coin, "getblockchaininfo"),
    queryFn: () => rpcGetBlockchainInfo(coin),
    enabled: !isLight && visible,
    refetchInterval: false,
    staleTime: 2_000,
  });

  const explorerBlocks = useQuery({
    queryKey: coinQueryKey(coin, "explorer-blocks", 10),
    queryFn: () => fetchExplorerBlocks(coin, 10),
    enabled: isLight && visible,
    staleTime: 5_000,
    refetchInterval: isLight && visible ? 5_000 : false,
  });

  const rpcHeight = blockchain.data?.blocks ?? 0;
  const lightHeight = isLight ? (lightServer.tipHeight ?? 0) : 0;
  const storeHeight = chainTip.tip?.height ?? 0;
  const effectiveStoreHeight = Math.max(storeHeight, lightHeight);
  const height = Math.max(rpcHeight, effectiveStoreHeight) || null;
  const hash =
    effectiveStoreHeight >= rpcHeight
      ? chainTip.tip?.hash
      : (blockchain.data?.bestblockhash ?? chainTip.tip?.hash);
  const time =
    effectiveStoreHeight >= rpcHeight ? (chainTip.tip?.time ?? 0) : 0;

  useEffect(() => {
    if (!isLight || lightHeight <= 0) return;
    const indexed = explorerBlocks.data?.find(
      (block) => block.height === lightHeight,
    );
    if (indexed?.hash && indexed.time > 0) {
      if (
        chainTip.tip?.hash === indexed.hash &&
        chainTip.tip.height === lightHeight &&
        chainTip.tip.time === indexed.time
      ) {
        return;
      }
      pushChainTip({
        coin,
        height: lightHeight,
        hash: indexed.hash,
        time: indexed.time,
        block: indexed,
      });
      return;
    }
    if (storeHeight >= lightHeight) return;
    const stubHash = `light-tip-${lightHeight}`;
    if (chainTip.tip?.hash === stubHash && chainTip.tip.height === lightHeight) {
      return;
    }

    pushChainTip({
      coin,
      height: lightHeight,
      hash: stubHash,
      time: 0,
    });
  }, [
    chainTip.tip?.hash,
    chainTip.tip?.height,
    chainTip.tip?.time,
    coin,
    explorerBlocks.data,
    isLight,
    lightHeight,
    storeHeight,
  ]);

  useEffect(() => {
    if (isLight || rpcHeight <= 0) return;
    const rpcHash = blockchain.data?.bestblockhash;
    if (!rpcHash) return;
    if (rpcHeight <= storeHeight) return;
    if (chainTip.tip?.hash === rpcHash) return;

    pushChainTip({
      coin,
      height: rpcHeight,
      hash: rpcHash,
      time: 0,
    });
  }, [
    blockchain.data?.bestblockhash,
    chainTip.tip?.hash,
    coin,
    isLight,
    rpcHeight,
    storeHeight,
  ]);

  return {
    height,
    hash,
    time,
    chainTip,
    blockchain,
    isLight,
  };
}
