// Copyright (c) 2026 The Vericonomy developers
// Distributed under the MIT software license, see the accompanying
// file COPYING or http://www.opensource.org/licenses/mit-license.php.
//
// Persistent banner shown at the top of AppShell whenever the wallet is
// pointed at the binarytest (DACE) network. The banner is intentionally
// loud — it's the visual safety guard that prevents users from confusing
// their test wallet with their real wallet.

import { Link } from 'react-router-dom';
import { BINARYTEST_ENABLED } from '@/lib/features';
import { useIsTestNetwork } from '@/lib/network-mode';

export function NetworkModeBanner() {
  const isTest = useIsTestNetwork();
  if (!BINARYTEST_ENABLED || !isTest) return null;

  return (
    <div className="w-full min-w-0 max-w-full border-b border-amber-500/50 bg-amber-500/20 px-4 py-2 text-xs text-amber-100">
      <div className="flex min-w-0 flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
        <div className="flex min-w-0 flex-wrap items-center gap-2">
          <span className="inline-block h-2 w-2 shrink-0 rounded-full bg-amber-400 animate-pulse" />
          <span className="shrink-0 font-semibold uppercase tracking-wide">Binarytest</span>
          <span className="min-w-0 text-amber-200/80">
            DACE test network — funds have no real value. Distinct datadirs and ports; cannot
            connect to mainnet peers.
          </span>
        </div>
        <div className="flex shrink-0 flex-wrap items-center gap-3">
          <Link
            to="/binary-chain"
            className="font-semibold underline underline-offset-2 hover:text-amber-50"
          >
            Binary Chain status
          </Link>
          <Link
            to="/settings"
            className="font-semibold underline underline-offset-2 hover:text-amber-50"
          >
            Switch back to mainnet
          </Link>
        </div>
      </div>
    </div>
  );
}
