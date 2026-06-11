import { useQuery } from '@tanstack/react-query';
import { useWindowVisible } from '@/hooks/useWindowVisible';
import {
  fetchMinerHashrateHistory,
  fetchMinerOverview,
  fetchMinerPayouts,
  fetchPoolPayoutSummary,
  fetchPoolStats,
  isPoolApiEnabled,
} from '@/lib/pool-api';

export function usePoolStatsQuery(enabled = true) {
  const visible = useWindowVisible();
  return useQuery({
    queryKey: ['pool', 'stats'],
    queryFn: fetchPoolStats,
    enabled: enabled && visible,
    refetchInterval: false,
    staleTime: 15_000,
    retry: 1,
  });
}

export function usePoolApiEnabledQuery() {
  return useQuery({
    queryKey: ['pool', 'enabled'],
    queryFn: isPoolApiEnabled,
    staleTime: 60_000,
  });
}

export function useMinerOverviewQuery(address: string | undefined, enabled = true) {
  const visible = useWindowVisible();
  const addr = address?.trim();
  return useQuery({
    queryKey: ['pool', 'miner', addr],
    queryFn: () => fetchMinerOverview(addr!),
    enabled: Boolean(enabled && visible && addr),
    refetchInterval: false,
    staleTime: 10_000,
    retry: 1,
  });
}

export function useMinerHashrateHistoryQuery(address: string | undefined, enabled = true) {
  const visible = useWindowVisible();
  const addr = address?.trim();
  return useQuery({
    queryKey: ['pool', 'hashrate-history', addr],
    queryFn: () => fetchMinerHashrateHistory(addr!),
    enabled: Boolean(enabled && visible && addr),
    refetchInterval: false,
    staleTime: 15_000,
    retry: 1,
  });
}

export function usePoolPayoutSummaryQuery(enabled = true) {
  const visible = useWindowVisible();
  return useQuery({
    queryKey: ['pool', 'payout-summary'],
    queryFn: fetchPoolPayoutSummary,
    enabled: enabled && visible,
    refetchInterval: false,
    staleTime: 15_000,
    retry: 1,
  });
}

export function useMinerPayoutsQuery(address: string | undefined, enabled = true) {
  const visible = useWindowVisible();
  const addr = address?.trim();
  return useQuery({
    queryKey: ['pool', 'miner-payouts', addr],
    queryFn: () => fetchMinerPayouts(addr!, 20, 0),
    enabled: Boolean(enabled && visible && addr),
    refetchInterval: false,
    staleTime: 10_000,
    retry: 1,
  });
}
