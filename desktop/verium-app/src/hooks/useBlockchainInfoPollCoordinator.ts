import { useQuery } from "@tanstack/react-query";
import { coinQueryKey } from "@/lib/coin/profile";
import { useUserPreferences } from "@/lib/user-preferences";
import { useWalletMode } from "@/hooks/useWalletMode";
import { useWindowVisible } from "@/hooks/useWindowVisible";
import { rpcGetBlockchainInfo, type BlockchainInfo } from "@/lib/rpc/client";

const CHAIN_POLL_MS = 30_000;
const CHAIN_SYNC_POLL_MS = 15_000;

function pollInterval(data: BlockchainInfo | undefined, visible: boolean) {
  if (!visible) return false;
  const syncing =
    data != null &&
    data.headers != null &&
    data.blocks != null &&
    data.headers > data.blocks + 1;
  return syncing || data?.initialblockdownload ? CHAIN_SYNC_POLL_MS : CHAIN_POLL_MS;
}

/**
 * Single writer for `getblockchaininfo` — other hooks must use `refetchInterval: false`.
 */
export function useBlockchainInfoPollCoordinator(): void {
  const visible = useWindowVisible();
  const { isLight } = useWalletMode();
  const prefs = useUserPreferences((s) => s.prefs);
  const enabled = !isLight;

  useQuery({
    queryKey: coinQueryKey("verium", "getblockchaininfo"),
    queryFn: () => rpcGetBlockchainInfo("verium"),
    enabled: enabled && prefs.verium_enabled !== false,
    refetchInterval: (q) => pollInterval(q.state.data ?? undefined, visible),
    staleTime: 10_000,
    gcTime: 30_000,
  });

  useQuery({
    queryKey: coinQueryKey("vericoin", "getblockchaininfo"),
    queryFn: () => rpcGetBlockchainInfo("vericoin"),
    enabled: enabled && prefs.vericoin_enabled !== false,
    refetchInterval: (q) => pollInterval(q.state.data ?? undefined, visible),
    staleTime: 10_000,
    gcTime: 30_000,
  });
}
