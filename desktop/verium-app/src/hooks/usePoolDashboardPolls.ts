import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { useWindowVisible } from "@/hooks/useWindowVisible";
import { POOL_DASHBOARD_POLL_MS } from "@/lib/mining-poll";

/**
 * Single invalidation loop for pool Supabase dashboard queries on the mining page.
 * Paused while the local veriumMiner sidecar is hashing (local hashrate needs no cloud polls).
 */
export function usePoolDashboardPolls(
  address: string | undefined,
  enabled: boolean,
): void {
  const visible = useWindowVisible();
  const queryClient = useQueryClient();
  const addr = address?.trim();

  useEffect(() => {
    if (!enabled || !visible || !addr) return;

    const tick = () => {
      void queryClient.invalidateQueries({ queryKey: ["pool", "miner", addr] });
      void queryClient.invalidateQueries({
        queryKey: ["pool", "hashrate-history", addr],
      });
      void queryClient.invalidateQueries({
        queryKey: ["pool", "miner-payouts", addr],
      });
    };

    tick();
    const id = window.setInterval(tick, POOL_DASHBOARD_POLL_MS);
    return () => window.clearInterval(id);
  }, [enabled, visible, addr, queryClient]);
}
